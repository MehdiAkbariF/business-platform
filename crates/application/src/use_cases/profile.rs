use std::sync::Arc;
use bytes::Bytes;
use domain::business::BusinessStatus;
use domain::profile::{BusinessHoursInterval, BusinessMedia, MediaStatus, MediaType, SocialPlatform};
use shared::{AttributeId, BusinessId, ClientMetadata, MediaId, UserId};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::ports::repositories::{
    AuditRepository, BusinessRepository, MembershipRepository, ProfileRepository, TaxonomyRepository,
};
use crate::ports::storage::ObjectStoragePort;
use crate::use_cases::business::{ContactDto, LocationDto};
use crate::use_cases::taxonomy::{BusinessCategoryDto, BusinessServiceDto};

#[derive(Serialize, ToSchema)]
pub struct MediaDto {
    pub id: MediaId,
    pub media_type: MediaType,
    pub url: String,
    pub width: i32,
    pub height: i32,
    pub alt_text: Option<String>,
    pub sort_order: i32,
}

#[derive(Serialize, ToSchema)]
pub struct AttributeItemDto {
    pub key: String,
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
pub struct SocialLinkDto {
    pub platform: SocialPlatform,
    pub url: String,
}

#[derive(Serialize, ToSchema)]
pub struct PublicPresentationDto {
    pub id: BusinessId,
    pub slug: String,
    pub name: String,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub timezone: String,
    pub primary_category: Option<String>,
    pub categories: Vec<BusinessCategoryDto>,
    pub services: Vec<BusinessServiceDto>,
    pub primary_location: Option<LocationDto>,
    pub locations: Vec<LocationDto>,
    pub contact: Option<ContactDto>,
    pub media: Vec<MediaDto>,
    pub hours: Vec<BusinessHoursInterval>,
    pub attributes: Vec<AttributeItemDto>,
    pub social_links: Vec<SocialLinkDto>,
    pub completeness_score: u8,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateProfileDetailsCommand {
    pub name: String,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SaveAttributesCommand {
    pub attributes: Vec<AttributePayload>,
}

#[derive(Deserialize, ToSchema)]
pub struct AttributePayload {
    pub attribute_id: AttributeId,
    pub value: serde_json::Value,
}

#[derive(Deserialize, ToSchema)]
pub struct SaveSocialLinksCommand {
    pub links: Vec<SocialLinkPayload>,
}

#[derive(Deserialize, ToSchema)]
pub struct SocialLinkPayload {
    pub platform: SocialPlatform,
    pub url: String,
}

pub fn calculate_completeness(
    has_desc: bool,
    has_location: bool,
    has_contact: bool,
    has_category: bool,
    has_media: bool,
    has_hours: bool,
) -> u8 {
    let mut score = 0;
    if has_desc { score += 20; }
    if has_location { score += 20; }
    if has_contact { score += 15; }
    if has_category { score += 15; }
    if has_media { score += 15; }
    if has_hours { score += 15; }
    score
}

pub async fn get_public_presentation(
    business_repo: Arc<dyn BusinessRepository>,
    tax_repo: Arc<dyn TaxonomyRepository>,
    profile_repo: Arc<dyn ProfileRepository>,
    storage_endpoint: &str,
    storage_bucket: &str,
    slug: &str,
) -> Result<PublicPresentationDto, AppError> {
    let business = business_repo
        .find_by_slug(slug)
        .await?
        .ok_or_else(|| AppError::NotFound("Business not found".to_string()))?;

    if business.status != BusinessStatus::Published {
        return Err(AppError::NotFound("Business not found".to_string()));
    }

    let categories = tax_repo.get_business_categories(business.id).await?;
    let primary_category = categories.iter().find(|c| c.is_primary).map(|c| c.name.clone());
    let cat_dtos = categories.into_iter().map(|c| BusinessCategoryDto {
        category_id: c.category_id,
        name: c.name,
        slug: c.slug,
        is_primary: c.is_primary,
    }).collect();

    let services = tax_repo.get_business_services(business.id).await?;
    let srv_dtos = services.into_iter().map(|s| BusinessServiceDto {
        service_id: s.service_id,
        name: s.name,
        slug: s.slug,
        is_active: s.is_active,
        sort_order: s.sort_order,
    }).collect();

    let locations = business_repo.get_locations(business.id).await?;
    let primary_location = locations.iter().find(|l| l.is_primary).map(|l| LocationDto {
        id: l.id,
        label: l.label.clone(),
        latitude: l.latitude,
        longitude: l.longitude,
        formatted_address: l.formatted_address.clone(),
        is_primary: l.is_primary,
    });
    let loc_dtos = locations.into_iter().map(|l| LocationDto {
        id: l.id,
        label: l.label,
        latitude: l.latitude,
        longitude: l.longitude,
        formatted_address: l.formatted_address,
        is_primary: l.is_primary,
    }).collect();

    let contact = business_repo.get_contact(business.id).await?.map(|c| ContactDto {
        phone: c.phone,
        mobile: c.mobile,
        email: c.email,
        website: c.website,
    });

    let raw_media = profile_repo.get_media(business.id).await?;
    let media_dtos: Vec<MediaDto> = raw_media
        .into_iter()
        .filter(|m| m.status == MediaStatus::Active)
        .map(|m| {
            let url = format!("{}/{}/{}", storage_endpoint, storage_bucket, m.storage_key);
            MediaDto {
                id: m.id,
                media_type: m.media_type,
                url,
                width: m.width,
                height: m.height,
                alt_text: m.alt_text,
                sort_order: m.sort_order,
            }
        })
        .collect();

    let hours = profile_repo.get_hours(business.id).await?;
    let attrs = profile_repo.get_attributes(business.id).await?;
    let attr_dtos = attrs.into_iter().map(|a| AttributeItemDto {
        key: a.key,
        name: a.name,
        value: a.value_json,
    }).collect();

    let links = profile_repo.get_social_links(business.id).await?;
    let link_dtos = links.into_iter().map(|l| SocialLinkDto {
        platform: l.platform,
        url: l.url,
    }).collect();

    let score = calculate_completeness(
        business.description.is_some(),
        primary_location.is_some(),
        contact.is_some(),
        primary_category.is_some(),
        !media_dtos.is_empty(),
        !hours.is_empty(),
    );

    Ok(PublicPresentationDto {
        id: business.id,
        slug: business.slug.as_str().to_string(),
        name: business.name,
        short_description: business.short_description,
        description: business.description,
        timezone: business.timezone,
        primary_category,
        categories: cat_dtos,
        services: srv_dtos,
        primary_location,
        locations: loc_dtos,
        contact,
        media: media_dtos,
        hours,
        attributes: attr_dtos,
        social_links: link_dtos,
        completeness_score: score,
    })
}

pub async fn upload_business_media(
    membership_repo: Arc<dyn MembershipRepository>,
    profile_repo: Arc<dyn ProfileRepository>,
    storage: Arc<dyn ObjectStoragePort>,
    audit_repo: Arc<dyn AuditRepository>,
    business_id: BusinessId,
    user_id: UserId,
    media_type: MediaType,
    file_bytes: Bytes,
    alt_text: Option<String>,
    metadata: ClientMetadata,
) -> Result<MediaDto, AppError> {
    let membership = membership_repo
        .find_membership(business_id, user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("You are not a member of this business".to_string()))?;

    if !membership.can_edit_profile() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let img_format = image::guess_format(&file_bytes)
        .map_err(|_| AppError::Validation("Invalid image format. Supported formats: JPEG, PNG, WebP".to_string()))?;

    let (mime_type, ext) = match img_format {
        image::ImageFormat::Jpeg => ("image/jpeg", "jpg"),
        image::ImageFormat::Png => ("image/png", "png"),
        image::ImageFormat::WebP => ("image/webp", "webp"),
        _ => return Err(AppError::Validation("Unsupported image format. SVG and unapproved types are rejected.".to_string())),
    };

    let dynamic_img = image::load_from_memory(&file_bytes)
        .map_err(|e| AppError::Validation(format!("Corrupted image data: {e}")))?;

    let width = dynamic_img.width() as i32;
    let height = dynamic_img.height() as i32;

    let mut cleaned_bytes: Vec<u8> = Vec::new();
    dynamic_img.write_to(&mut std::io::Cursor::new(&mut cleaned_bytes), image::ImageFormat::Jpeg)
        .map_err(|e| AppError::internal(anyhow::anyhow!(e)))?;

    let media_id = MediaId::new();
    let storage_key = format!("businesses/{}/{}_{}.{}", business_id.0, media_type.to_string().to_lowercase(), media_id.0, ext);

    storage.put_object(&storage_key, Bytes::from(cleaned_bytes.clone()), mime_type).await?;

    let media = BusinessMedia {
        id: media_id,
        business_id,
        media_type,
        storage_key: storage_key.clone(),
        mime_type: mime_type.to_string(),
        size_bytes: cleaned_bytes.len() as i64,
        width,
        height,
        alt_text: alt_text.clone(),
        sort_order: 0,
        status: MediaStatus::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    profile_repo.save_media(&media).await?;

    let _ = audit_repo.record(
        Some(user_id),
        "BUSINESS_MEDIA_UPLOADED",
        metadata.ip_address,
        metadata.user_agent,
        Some(serde_json::json!({ "business_id": business_id.to_string(), "media_id": media_id.to_string(), "media_type": media_type.to_string() })),
    ).await;

    Ok(MediaDto {
        id: media_id,
        media_type,
        url: storage_key,
        width,
        height,
        alt_text,
        sort_order: 0,
    })
}