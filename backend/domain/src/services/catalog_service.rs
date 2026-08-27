use crate::repositories::ProductRepository;

pub struct CatalogService<P>
where
    P: ProductRepository,
{
    product_repo: P,
}

impl<P> CatalogService<P>
where
    P: ProductRepository,
{
    pub fn new(product_repo: P) -> Self {
        Self { product_repo }
    }

    // Catalog business logic placeholder
    pub async fn apply_discount_rules(
        &self,
        product_id: uuid::Uuid,
    ) -> Result<(), crate::errors::DomainError> {
        let _product = self.product_repo.get_product_by_id(product_id).await?
            .ok_or_else(|| crate::errors::DomainError::ProductNotFound)?;
        Ok(())
    }
}
