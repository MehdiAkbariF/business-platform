use std::sync::Arc;
use domain::seo::{PageMetadata, StructuredAddress, StructuredDataJsonLd, StructuredGeo};
use crate::errors::AppError;
use crate::ports::repositories::{BusinessRepository, TaxonomyRepository};
use crate::ports::seo::{BusinessSeoPageDto, CategoryLocationLandingDto, SeoRepository};
use crate::use_cases::business::get_public_profile;

pub async fn get_business_seo_page(
    seo_repo: Arc<dyn SeoRepository>,
    business_repo: Arc<dyn BusinessRepository>,
    _tax_repo: Arc<dyn TaxonomyRepository>,
    public_origin: &str,
    slug: &str,
) -> Result<BusinessSeoPageDto, AppError> {
    // 1. Check for 301 Permanent Redirect
    if let Some(target_slug) = seo_repo.find_redirect(slug).await? {
        return Ok(BusinessSeoPageDto {
            is_redirect: true,
            redirect_target: Some(format!("{}/businesses/{}", public_origin, target_slug)),
            metadata: PageMetadata {
                title: "Redirecting...".to_string(),
                description: String::new(),
                canonical_url: format!("{}/businesses/{}", public_origin, target_slug),
                robots: "noindex, follow".to_string(),
                og_title: String::new(),
                og_description: String::new(),
                og_image: None,
                og_type: "website".to_string(),
            },
            structured_data: None,
            profile: None,
            related_businesses_slugs: Vec::new(),
        });
    }

    // 2. Fetch Published Business Profile
    let profile = get_public_profile(business_repo.clone(), slug).await?;

    let canonical_url = format!("{}/businesses/{}", public_origin, profile.slug);
    let title = format!("{} | مشخصات، آدرس و تماس", profile.name);
    let description = profile.short_description.clone().unwrap_or_else(|| format!("اطلاعات تماس، ساعات کاری و خدمات {} در پلتفرم کشف کسب‌وکار", profile.name));

    let structured_data = StructuredDataJsonLd {
        context: "https://schema.org".to_string(),
        schema_type: "LocalBusiness".to_string(),
        name: profile.name.clone(),
        description: profile.description.clone(),
        url: canonical_url.clone(),
        telephone: profile.contact.as_ref().and_then(|c| c.phone.clone().or(c.mobile.clone())),
        address: profile.primary_location.as_ref().map(|l| StructuredAddress {
            street_address: l.formatted_address.clone(),
            address_locality: "تهران".to_string(),
            address_region: "تهران".to_string(),
            address_country: "IR".to_string(),
        }),
        geo: profile.primary_location.as_ref().map(|l| StructuredGeo {
            latitude: l.latitude,
            longitude: l.longitude,
        }),
    };

    Ok(BusinessSeoPageDto {
        is_redirect: false,
        redirect_target: None,
        metadata: PageMetadata {
            title: title.clone(),
            description: description.clone(),
            canonical_url: canonical_url.clone(),
            robots: "index, follow".to_string(),
            og_title: title,
            og_description: description,
            og_image: None,
            og_type: "business.business".to_string(),
        },
        structured_data: Some(structured_data),
        profile: Some(profile),
        related_businesses_slugs: Vec::new(),
    })
}

pub async fn get_category_location_landing_page(
    seo_repo: Arc<dyn SeoRepository>,
    tax_repo: Arc<dyn TaxonomyRepository>,
    _business_repo: Arc<dyn BusinessRepository>,
    public_origin: &str,
    min_inventory: usize,
    city: &str,
    category_slug: &str,
) -> Result<CategoryLocationLandingDto, AppError> {
    let category = tax_repo.find_category_by_slug(category_slug).await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let count = seo_repo.count_published_businesses_in_category_city(category.id, city).await?;
    let is_indexable = count >= min_inventory;

    let canonical_url = format!("{}/discover/{}/{}", public_origin, city, category_slug);
    let title = format!("بهترین {} در {} | لیست کسب‌وکارها", category.name, city);
    let description = format!("لیست و اطلاعات کامل بهترین {} در شهر {} به همراه آدرس و نظرات مشتریان", category.name, city);

    Ok(CategoryLocationLandingDto {
        city: city.to_string(),
        category_name: category.name,
        category_slug: category_slug.to_string(),
        business_count: count,
        is_indexable,
        metadata: PageMetadata {
            title: title.clone(),
            description: description.clone(),
            canonical_url,
            robots: if is_indexable { "index, follow" } else { "noindex, follow" }.to_string(),
            og_title: title,
            og_description: description,
            og_image: None,
            og_type: "website".to_string(),
        },
        featured_businesses: Vec::new(),
    })
}

pub async fn generate_sitemap_index_xml(public_origin: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <sitemap><loc>{}/sitemaps/businesses.xml</loc></sitemap>
    <sitemap><loc>{}/sitemaps/categories.xml</loc></sitemap>
    <sitemap><loc>{}/sitemaps/locations.xml</loc></sitemap>
</sitemapindex>"#,
        public_origin, public_origin, public_origin
    )
}

pub async fn generate_businesses_sitemap_xml(seo_repo: Arc<dyn SeoRepository>, public_origin: &str) -> Result<String, AppError> {
    let entries = seo_repo.get_indexable_businesses_sitemap(1000, 0).await?;
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for entry in entries {
        xml.push_str(&format!(
            "  <url>\n    <loc>{}/businesses/{}</loc>\n    <lastmod>{}</lastmod>\n    <changefreq>weekly</changefreq>\n    <priority>0.8</priority>\n  </url>\n",
            public_origin, entry.loc, entry.lastmod.format("%Y-%m-%d")
        ));
    }
    xml.push_str("</urlset>");
    Ok(xml)
}