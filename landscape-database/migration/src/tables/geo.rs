use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum GeoSiteConfigs {
    #[sea_orm(iden = "geo_site_configs")]
    Table,
    Id,
    UpdateAt,
    Url,
    Name,
    Enable,
    NextUpdateAt,
    GeoKeys,
}

#[derive(DeriveIden)]
pub enum GeoIpConfigs {
    #[sea_orm(iden = "geo_ip_configs")]
    Table,
    Id,
    UpdateAt,
    Url,
    Name,
    Enable,
    NextUpdateAt,
}
