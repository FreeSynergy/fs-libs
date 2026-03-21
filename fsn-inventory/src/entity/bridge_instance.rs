//! Sea-ORM entity for the `bridge_instances` table.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bridge_instances")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub bridge_id: String,
    pub role: String,
    pub service_instance: String,
    pub api_base_url: String,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::service_instance::Entity",
        from = "Column::ServiceInstance",
        to = "super::service_instance::Column::Id"
    )]
    ServiceInstance,
}

impl Related<super::service_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServiceInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
