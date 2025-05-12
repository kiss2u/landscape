# Landscape Database

# seaORM Column Type 
> https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-structure/#column-type

The column type will be derived automatically with the following mapping:

> SQL Server (MSSQL) backend
> The column type mapping of MSSQL can be found [here](https://www.sea-ql.org/SeaORM-X/docs/generate-entity/entity-structure/).


For the mappings of Rust primitive data types.

| Rust type | Database Type <br/> ([`ColumnType`](https://docs.rs/sea-orm/*/sea_orm/entity/enum.ColumnType.html)) | SQLite <br/> datatype | MySQL <br/> datatype | PostgreSQL <br/> datatype |
| --------- | --------- | --------- | --------- | --------- |
| `String` | Char | char | char | char |
| `String` | String | varchar | varchar | varchar |
| `i8` | TinyInteger | tinyint | tinyint | char |
| `u8` | TinyUnsigned | tinyint  | tinyint unsigned | N/A |
| `i16` | SmallInteger | smallint | smallint | smallint |
| `u16` | SmallUnsigned | smallint | smallint unsigned | N/A |
| `i32` | Integer | integer | int | integer |
| `u32` | Unsigned | integer | int unsigned | N/A |
| `i64` | BigInteger | bigint | bigint | bigint |
| `u64` | BigUnsigned | bigint | bigint unsigned | N/A |
| `f32` | Float | float | float | real |
| `f64` | Double | double | double | double precision |
| `bool` | Boolean | boolean | bool | bool |
| `Vec<u8>` | Binary | blob | blob | bytea |

For the mappings of Rust non-primitive data types. You can check [`entity/prelude.rs`](https://github.com/SeaQL/sea-orm/blob/master/src/entity/prelude.rs) for all of the reexported types.

| Rust type | Database Type <br/> ([`ColumnType`](https://docs.rs/sea-orm/*/sea_orm/entity/enum.ColumnType.html)) | SQLite <br/> datatype | MySQL <br/> datatype | PostgreSQL <br/> datatype |
| --------- | --------- | --------- | --------- | --------- |
| `Date`: chrono::NaiveDate <br/>`TimeDate`: time::Date | Date | date_text | date | date |
| `Time`: chrono::NaiveTime <br/>`TimeTime`: time::Time | Time | time_text | time | time |
| `DateTime`: chrono::NaiveDateTime <br/>`TimeDateTime`: time::PrimitiveDateTime | DateTime | datetime_text | datetime | timestamp |
| `DateTimeLocal`: chrono::DateTime&lt;Local&gt; <br/>`DateTimeUtc`: chrono::DateTime&lt;Utc&gt; | Timestamp | timestamp_text | timestamp | N/A |
| `DateTimeWithTimeZone`: chrono::DateTime&lt;FixedOffset&gt; <br/>`TimeDateTimeWithTimeZone`: time::OffsetDateTime | TimestampWithTimeZone | timestamp_with_timezone_text | timestamp | timestamp with time zone |
| `Uuid`: uuid::Uuid, uuid::fmt::Braced, uuid::fmt::Hyphenated, uuid::fmt::Simple, uuid::fmt::Urn | Uuid | uuid_text | binary(16) | uuid |
| `Json`: serde_json::Value | Json | json_text | json | json |
| `Decimal`: rust_decimal::Decimal | Decimal | real | decimal | decimal |

You can override the default mappings between a Rust type and `ColumnType` by the `column_type` attribute.