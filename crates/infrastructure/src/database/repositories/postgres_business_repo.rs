use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::business::{Business, BusinessSlug, BusinessStatus};
use domain::contact::BusinessContact;
use domain::location::{BusinessLocation, LocationAccuracy};
use domain::membership::BusinessMembership;
use shared::{BusinessId, LocationId, UserId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::BusinessRepository;
use uuid::Uuid;

pub struct PostgresBusinessRepository {
    pool: PgPool,
}

impl PostgresBusinessRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct BusinessRow {
    id: Uuid,
    slug: String,
    name: String,
    short_description: Option<String>,
    description: Option<String>,
    timezone: String,
    status: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
}

impl TryFrom<BusinessRow> for Business {
    type Error = AppError;
    fn try_from(row: BusinessRow) -> Result<Self, Self::Error> {
        let slug = BusinessSlug::parse(&row.slug).map_err(|e| AppError::internal(anyhow::anyhow!(e)))?;
        let status = match row.status.as_str() {
            "DRAFT" => BusinessStatus::Draft,
            "PENDING_REVIEW" => BusinessStatus::PendingReview,
            "PUBLISHED" => BusinessStatus::Published,
            "SUSPENDED" => BusinessStatus::Suspended,
            "ARCHIVED" => BusinessStatus::Archived,
            _ => BusinessStatus::Draft,
        };

        Ok(Business {
            id: BusinessId::from_uuid(row.id),
            slug,
            name: row.name,
            short_description: row.short_description,
            description: row.description,
            timezone: row.timezone,
            status,
            created_by: UserId::from_uuid(row.created_by),
            created_at: row.created_at,
            updated_at: row.updated_at,
            published_at: row.published_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct LocationRow {
    id: Uuid,
    business_id: Uuid,
    label: String,
    lat: f64,
    lon: f64,
    country: String,
    province: String,
    city: String,
    district: Option<String>,
    street: String,
    postal_code: Option<String>,
    formatted_address: String,
    accuracy: String,
    is_primary: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<LocationRow> for BusinessLocation {
    fn from(r: LocationRow) -> Self {
        let accuracy = match r.accuracy.as_str() {
            "EXACT" => LocationAccuracy::Exact,
            _ => LocationAccuracy::Approximate,
        };
        BusinessLocation {
            id: LocationId::from_uuid(r.id),
            business_id: BusinessId::from_uuid(r.business_id),
            label: r.label,
            latitude: r.lat,
            longitude: r.lon,
            country: r.country,
            province: r.province,
            city: r.city,
            district: r.district,
            street: r.street,
            postal_code: r.postal_code,
            formatted_address: r.formatted_address,
            accuracy,
            is_primary: r.is_primary,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ContactRow {
    business_id: Uuid,
    phone: Option<String>,
    mobile: Option<String>,
    email: Option<String>,
    website: Option<String>,
    preferred_contact_method: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[async_trait]
impl BusinessRepository for PostgresBusinessRepository {
    async fn create_with_owner(&self, business: &Business, owner_membership: &BusinessMembership) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        sqlx::query(
            "INSERT INTO businesses (id, slug, name, short_description, description, timezone, status, created_by, created_at, updated_at, published_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7::business_status, $8, $9, $10, $11)"
        )
        .bind(business.id.0)
        .bind(business.slug.as_str())
        .bind(&business.name)
        .bind(&business.short_description)
        .bind(&business.description)
        .bind(&business.timezone)
        .bind(business.status.to_string())
        .bind(business.created_by.0)
        .bind(business.created_at)
        .bind(business.updated_at)
        .bind(business.published_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("A business with this slug already exists".to_string());
                }
            }
            AppError::internal(e)
        })?;

        sqlx::query(
            "INSERT INTO business_memberships (id, business_id, user_id, role, status, created_at, updated_at) VALUES ($1, $2, $3, $4::membership_role, $5::membership_status, $6, $7)"
        )
        .bind(owner_membership.id.0)
        .bind(owner_membership.business_id.0)
        .bind(owner_membership.user_id.0)
        .bind(owner_membership.role.to_string())
        .bind(owner_membership.status.to_string())
        .bind(owner_membership.created_at)
        .bind(owner_membership.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(e))?;

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn find_by_id(&self, id: BusinessId) -> Result<Option<Business>, AppError> {
        let row = sqlx::query_as::<_, BusinessRow>(
            "SELECT id, slug, name, short_description, description, timezone, status::text, created_by, created_at, updated_at, published_at FROM businesses WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(Business::try_from).transpose()
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Business>, AppError> {
        let row = sqlx::query_as::<_, BusinessRow>(
            "SELECT id, slug, name, short_description, description, timezone, status::text, created_by, created_at, updated_at, published_at FROM businesses WHERE LOWER(slug) = LOWER($1)"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(Business::try_from).transpose()
    }

    async fn update_profile(&self, id: BusinessId, name: &str, short_desc: Option<&str>, description: Option<&str>, timezone: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE businesses SET name = $1, short_description = $2, description = $3, timezone = $4, updated_at = NOW() WHERE id = $5")
            .bind(name)
            .bind(short_desc)
            .bind(description)
            .bind(timezone)
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn update_status(&self, id: BusinessId, status: BusinessStatus) -> Result<(), AppError> {
        sqlx::query("UPDATE businesses SET status = $1::business_status, updated_at = NOW() WHERE id = $2")
            .bind(status.to_string())
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn count_primary_locations(&self, business_id: BusinessId) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM business_locations WHERE business_id = $1 AND is_primary = TRUE")
            .bind(business_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(count.0)
    }

    async fn add_location(&self, l: &BusinessLocation) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO business_locations (id, business_id, label, geom, country, province, city, district, street, postal_code, formatted_address, accuracy, is_primary, created_at, updated_at) 
             VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5), 4326), $6, $7, $8, $9, $10, $11, $12, $13::location_accuracy, $14, $15, $16)"
        )
        .bind(l.id.0)
        .bind(l.business_id.0)
        .bind(&l.label)
        .bind(l.longitude)
        .bind(l.latitude)
        .bind(&l.country)
        .bind(&l.province)
        .bind(&l.city)
        .bind(&l.district)
        .bind(&l.street)
        .bind(&l.postal_code)
        .bind(&l.formatted_address)
        .bind(l.accuracy.to_string())
        .bind(l.is_primary)
        .bind(l.created_at)
        .bind(l.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn get_locations(&self, business_id: BusinessId) -> Result<Vec<BusinessLocation>, AppError> {
        let rows = sqlx::query_as::<_, LocationRow>(
            "SELECT id, business_id, label, ST_Y(geom) as lat, ST_X(geom) as lon, country, province, city, district, street, postal_code, formatted_address, accuracy::text, is_primary, created_at, updated_at 
             FROM business_locations WHERE business_id = $1 ORDER BY is_primary DESC, created_at ASC"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(BusinessLocation::from).collect())
    }

    async fn get_primary_location(&self, business_id: BusinessId) -> Result<Option<BusinessLocation>, AppError> {
        let row = sqlx::query_as::<_, LocationRow>(
            "SELECT id, business_id, label, ST_Y(geom) as lat, ST_X(geom) as lon, country, province, city, district, street, postal_code, formatted_address, accuracy::text, is_primary, created_at, updated_at 
             FROM business_locations WHERE business_id = $1 AND is_primary = TRUE LIMIT 1"
        )
        .bind(business_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(BusinessLocation::from))
    }

    async fn save_contact(&self, c: &BusinessContact) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO business_contacts (business_id, phone, mobile, email, website, preferred_contact_method, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (business_id) DO UPDATE SET 
             phone = EXCLUDED.phone, mobile = EXCLUDED.mobile, email = EXCLUDED.email, website = EXCLUDED.website, preferred_contact_method = EXCLUDED.preferred_contact_method, updated_at = NOW()"
        )
        .bind(c.business_id.0)
        .bind(&c.phone)
        .bind(&c.mobile)
        .bind(&c.email)
        .bind(&c.website)
        .bind(&c.preferred_contact_method)
        .bind(c.created_at)
        .bind(c.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn get_contact(&self, business_id: BusinessId) -> Result<Option<BusinessContact>, AppError> {
        let row = sqlx::query_as::<_, ContactRow>(
            "SELECT business_id, phone, mobile, email, website, preferred_contact_method, created_at, updated_at FROM business_contacts WHERE business_id = $1"
        )
        .bind(business_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(row.map(|r| BusinessContact {
            business_id: BusinessId::from_uuid(r.business_id),
            phone: r.phone,
            mobile: r.mobile,
            email: r.email,
            website: r.website,
            preferred_contact_method: r.preferred_contact_method,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }
}