use fake::{
    Fake,
    faker::commerce::pt_br::{CommerceProduct, CommerceProductDescription, CommerceProductPrice},
};
use sea_orm::{
    ActiveValue::Set, Database, DeriveEntityModel, DerivePrimaryKey, DeriveRelation, EntityTrait,
    EnumIter, PrimaryKeyTrait,
};

const PRODUCT_COUNT: usize = 40_000;
const BATCH_SIZE: usize = 1_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(database_url).await?;

    let mut inserted = 0;

    for batch_start in (0..PRODUCT_COUNT).step_by(BATCH_SIZE) {
        let batch_size = (PRODUCT_COUNT - batch_start).min(BATCH_SIZE);
        let products = (0..batch_size).map(|_| fake_product()).collect::<Vec<_>>();

        Entity::insert_many(products).exec(&db).await?;
        inserted += batch_size;
    }

    println!("Seeded {inserted} products");

    Ok(())
}

fn fake_product() -> ActiveModel {
    let brand = CommerceProduct().fake::<String>();
    let description = CommerceProductDescription().fake::<String>();
    let price = CommerceProductPrice(1.0..40.0).fake::<f64>();

    ActiveModel {
        name: Set(brand),
        description: Set(Some(description)),
        price_cents: Set(price.round() as i32),
        stock: Set((0..500).fake::<i32>()),
        ..Default::default()
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub stock: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl sea_orm::ActiveModelBehavior for ActiveModel {}
