use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum NetIfaceConfigs {
    Table,
    Name, // 主键
    CreateDevType,
    ControllerName,
    ZoneType,
    EnableInBoot,
    WifiMode,
    XpsRps,
    UpdateAt,
}
