//! `SeaORM` Entity. Generated by sea-orm-codegen 1.0.0-rc.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "url")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub url: String,
    pub shady: String,
    pub created_at: DateTime,
    pub ip: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}