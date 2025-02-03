use std::{fs::OpenOptions, io::Write, path::Path};

use landscape_common::error::{LdError, LdResult};

const INIT_FILE_NAME: &'static str = "landscape_init.toml";

const INIT_LOCK_FILE_NAME: &'static str = "landscape_init.lock";
const INIT_LOCK_FILE_CONTENT: &'static str = r#"
⚠ 警告 ⚠
如果您不知道删除这个文件的操作是否正确, 请不要删除这个文件.
此文件用于确定当前的 Landscape Router 是否已经初始化.
删除后可能导致重新初始化.

⚠ WARNING ⚠
If you are not sure whether deleting this file is correct, please do not delete this file.
This file is used to determine whether the current Landscape Router has been initialized.
Deleting it may cause reinitialization.
"#;
/// 返回是否进行初始化操作  
/// true: 需要进行初始化  
/// false: 无需进行初始化  
/// Err: 出现错误退出  
pub fn boot_check<P: AsRef<Path>>(path: P) -> LdResult<bool> {
    let path = path.as_ref().join(INIT_LOCK_FILE_NAME);

    // 1. 先检查文件存不存在, 不存在 创建一个 Lock 文件
    if !path.exists() {
        let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(&path)?;
        file.write_all(INIT_LOCK_FILE_CONTENT.as_bytes())?;
        return Ok(true);
    }

    if path.is_file() {
        return Ok(false);
    }

    Err(LdError::Boot("check boot lock file faile: is not a file".to_string()))
}
