use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::taxonomy::{Category, Service, TaxonomySlug, TaxonomyStatus};
use shared::{BusinessId, CategoryId, ServiceId};
use sqlx::PgPool;
use application::errors::AppError;
use application::ports::repositories::{BusinessCategoryRow, BusinessServiceRow, TaxonomyRepository};
use uuid::Uuid;

pub struct PostgresTaxonomyRepository {
    pool: PgPool,
}

impl PostgresTaxonomyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    normalized_name: String,
    slug: String,
    description: Option<String>,
    status: String,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<CategoryRow> for Category {
    type Error = AppError;
    fn try_from(r: CategoryRow) -> Result<Self, Self::Error> {
        let slug = TaxonomySlug::parse(&r.slug).map_err(|e| AppError::internal(anyhow::anyhow!(e)))?;
        let status = match r.status.as_str() {
            "ACTIVE" => TaxonomyStatus::Active,
            _ => TaxonomyStatus::Inactive,
        };
        Ok(Category {
            id: CategoryId::from_uuid(r.id),
            parent_id: r.parent_id.map(CategoryId::from_uuid),
            name: r.name,
            normalized_name: r.normalized_name,
            slug,
            description: r.description,
            status,
            sort_order: r.sort_order,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ServiceRow {
    id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    normalized_name: String,
    slug: String,
    description: Option<String>,
    status: String,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ServiceRow> for Service {
    type Error = AppError;
    fn try_from(r: ServiceRow) -> Result<Self, Self::Error> {
        let slug = TaxonomySlug::parse(&r.slug).map_err(|e| AppError::internal(anyhow::anyhow!(e)))?;
        let status = match r.status.as_str() {
            "ACTIVE" => TaxonomyStatus::Active,
            _ => TaxonomyStatus::Inactive,
        };
        Ok(Service {
            id: ServiceId::from_uuid(r.id),
            parent_id: r.parent_id.map(ServiceId::from_uuid),
            name: r.name,
            normalized_name: r.normalized_name,
            slug,
            description: r.description,
            status,
            sort_order: r.sort_order,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[async_trait]
impl TaxonomyRepository for PostgresTaxonomyRepository {
    async fn list_active_categories(&self) -> Result<Vec<Category>, AppError> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            "SELECT id, parent_id, name, normalized_name, slug, description, status::text, sort_order, created_at, updated_at 
             FROM categories WHERE status = 'ACTIVE' ORDER BY sort_order ASC, name ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        rows.into_iter().map(Category::try_from).collect()
    }

    async fn find_category_by_id(&self, id: CategoryId) -> Result<Option<Category>, AppError> {
        let row = sqlx::query_as::<_, CategoryRow>(
            "SELECT id, parent_id, name, normalized_name, slug, description, status::text, sort_order, created_at, updated_at 
             FROM categories WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(Category::try_from).transpose()
    }

    async fn find_category_by_slug(&self, slug: &str) -> Result<Option<Category>, AppError> {
        let row = sqlx::query_as::<_, CategoryRow>(
            "SELECT id, parent_id, name, normalized_name, slug, description, status::text, sort_order, created_at, updated_at 
             FROM categories WHERE LOWER(slug) = LOWER($1)"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(Category::try_from).transpose()
    }

    async fn calculate_category_depth(&self, id: CategoryId) -> Result<usize, AppError> {
        let count: (i64,) = sqlx::query_as(
            "WITH RECURSIVE cat_tree AS (
                SELECT id, parent_id, 1 as depth FROM categories WHERE id = $1
                UNION ALL
                SELECT c.id, c.parent_id, ct.depth + 1 FROM categories c
                INNER JOIN cat_tree ct ON c.id = ct.parent_id
            ) SELECT MAX(depth) FROM cat_tree"
        )
        .bind(id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(count.0 as usize)
    }

    async fn list_active_services(&self) -> Result<Vec<Service>, AppError> {
        let rows = sqlx::query_as::<_, ServiceRow>(
            "SELECT id, parent_id, name, normalized_name, slug, description, status::text, sort_order, created_at, updated_at 
             FROM services WHERE status = 'ACTIVE' ORDER BY sort_order ASC, name ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        rows.into_iter().map(Service::try_from).collect()
    }

    async fn find_service_by_id(&self, id: ServiceId) -> Result<Option<Service>, AppError> {
        let row = sqlx::query_as::<_, ServiceRow>(
            "SELECT id, parent_id, name, normalized_name, slug, description, status::text, sort_order, created_at, updated_at 
             FROM services WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(Service::try_from).transpose()
    }

    async fn find_service_by_slug(&self, slug: &str) -> Result<Option<Service>, AppError> {
        let row = sqlx::query_as::<_, ServiceRow>(
            "SELECT id, parent_id, name, normalized_name, slug, description, status::text, sort_order, created_at, updated_at 
             FROM services WHERE LOWER(slug) = LOWER($1)"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        row.map(Service::try_from).transpose()
    }

    async fn is_service_compatible_with_categories(&self, service_id: ServiceId, category_ids: &[CategoryId]) -> Result<bool, AppError> {
        let uuids: Vec<Uuid> = category_ids.iter().map(|c| c.0).collect();
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM category_services WHERE service_id = $1 AND category_id = ANY($2)"
        )
        .bind(service_id.0)
        .bind(&uuids)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(count.0 > 0)
    }

    async fn get_business_categories(&self, business_id: BusinessId) -> Result<Vec<BusinessCategoryRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            category_id: Uuid,
            name: String,
            slug: String,
            is_primary: bool,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT bc.category_id, c.name, c.slug, bc.is_primary 
             FROM business_categories bc
             INNER JOIN categories c ON bc.category_id = c.id
             WHERE bc.business_id = $1
             ORDER BY bc.is_primary DESC, c.name ASC"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessCategoryRow {
            category_id: CategoryId::from_uuid(r.category_id),
            name: r.name,
            slug: r.slug,
            is_primary: r.is_primary,
        }).collect())
    }

    async fn add_business_category(&self, business_id: BusinessId, category_id: CategoryId, is_primary: bool) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        if is_primary {
            sqlx::query("UPDATE business_categories SET is_primary = FALSE WHERE business_id = $1")
                .bind(business_id.0)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::internal(e))?;
        }

        sqlx::query(
            "INSERT INTO business_categories (business_id, category_id, is_primary, created_at) VALUES ($1, $2, $3, NOW())"
        )
        .bind(business_id.0)
        .bind(category_id.0)
        .bind(is_primary)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("Category is already assigned to this business".to_string());
                }
            }
            AppError::internal(e)
        })?;

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn remove_business_category(&self, business_id: BusinessId, category_id: CategoryId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM business_categories WHERE business_id = $1 AND category_id = $2")
            .bind(business_id.0)
            .bind(category_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn set_primary_category(&self, business_id: BusinessId, category_id: CategoryId) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(e))?;

        // Clear existing primary
        sqlx::query("UPDATE business_categories SET is_primary = FALSE WHERE business_id = $1")
            .bind(business_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        // Set target as primary
        let res = sqlx::query("UPDATE business_categories SET is_primary = TRUE WHERE business_id = $1 AND category_id = $2")
            .bind(business_id.0)
            .bind(category_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(e))?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Category is not assigned to this business".to_string()));
        }

        tx.commit().await.map_err(|e| AppError::internal(e))?;
        Ok(())
    }

    async fn count_business_categories(&self, business_id: BusinessId) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM business_categories WHERE business_id = $1")
            .bind(business_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(count.0)
    }

    async fn has_primary_category(&self, business_id: BusinessId) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM business_categories WHERE business_id = $1 AND is_primary = TRUE")
            .bind(business_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(count.0 > 0)
    }

    async fn get_business_services(&self, business_id: BusinessId) -> Result<Vec<BusinessServiceRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            service_id: Uuid,
            name: String,
            slug: String,
            is_active: bool,
            sort_order: i32,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT bs.service_id, s.name, s.slug, bs.is_active, bs.sort_order 
             FROM business_services bs
             INNER JOIN services s ON bs.service_id = s.id
             WHERE bs.business_id = $1
             ORDER BY bs.sort_order ASC, s.name ASC"
        )
        .bind(business_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e))?;

        Ok(rows.into_iter().map(|r| BusinessServiceRow {
            service_id: ServiceId::from_uuid(r.service_id),
            name: r.name,
            slug: r.slug,
            is_active: r.is_active,
            sort_order: r.sort_order,
        }).collect())
    }

    async fn add_business_service(&self, business_id: BusinessId, service_id: ServiceId, sort_order: i32) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO business_services (business_id, service_id, is_active, sort_order, created_at, updated_at) VALUES ($1, $2, TRUE, $3, NOW(), NOW())"
        )
        .bind(business_id.0)
        .bind(service_id.0)
        .bind(sort_order)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("Service is already assigned to this business".to_string());
                }
            }
            AppError::internal(e)
        })?;

        Ok(())
    }

    async fn remove_business_service(&self, business_id: BusinessId, service_id: ServiceId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM business_services WHERE business_id = $1 AND service_id = $2")
            .bind(business_id.0)
            .bind(service_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(())
    }

    async fn count_business_services(&self, business_id: BusinessId) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM business_services WHERE business_id = $1")
            .bind(business_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(e))?;

        Ok(count.0)
    }
}