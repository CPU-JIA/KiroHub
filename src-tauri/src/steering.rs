// Steering 管理（读取/编辑 ~/.kiro/steering/*.md）

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteeringFile {
    pub file_name: String,
    pub content: String,
    pub size: u64,
    pub modified_at: Option<String>,
}

pub struct SteeringManager;

impl SteeringManager {
    /// 验证文件名，防止路径遍历攻击
    fn validate_file_name(file_name: &str) -> Result<(), String> {
        // 检查空文件名
        if file_name.is_empty() {
            return Err("文件名不能为空".to_string());
        }

        let path = std::path::Path::new(file_name);

        // 检查绝对路径
        if path.is_absolute() {
            return Err("不允许使用绝对路径".to_string());
        }

        // 检查危险的路径组件（如 .. 或根目录）
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err("文件名中不允许包含 '..'".to_string());
                }
                Component::RootDir => {
                    return Err("不允许使用根目录路径".to_string());
                }
                Component::Prefix(_) => {
                    return Err("不允许使用路径前缀".to_string());
                }
                _ => {}
            }
        }

        // 检查是否只是一个简单的文件名（不包含目录分隔符）
        if path.components().count() > 1 {
            return Err("文件名不能包含目录路径".to_string());
        }

        // 检查文件扩展名必须是 .md
        if path.extension().map(|e| e != "md").unwrap_or(true) {
            return Err("文件必须是 .md 格式".to_string());
        }

        Ok(())
    }

    /// 获取 steering 目录路径
    pub fn steering_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".kiro").join("steering"))
    }

    /// 读取所有 steering 文件列表
    pub fn load_all() -> Result<Vec<SteeringFile>, String> {
        let dir = Self::steering_dir().ok_or("无法获取用户目录")?;
        
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut files = vec![];
        
        for entry in fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                let file_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                
                let metadata = fs::metadata(&path).ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified_at = metadata
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                        datetime.format("%Y/%m/%d %H:%M:%S").to_string()
                    });
                
                let content = fs::read_to_string(&path).unwrap_or_default();
                
                files.push(SteeringFile {
                    file_name,
                    content,
                    size,
                    modified_at,
                });
            }
        }
        
        Ok(files)
    }

    /// 读取单个 steering 文件
    pub fn load(file_name: &str) -> Result<SteeringFile, String> {
        // 验证文件名，防止路径遍历
        Self::validate_file_name(file_name)?;

        let dir = Self::steering_dir().ok_or("无法获取用户目录")?;
        let path = dir.join(file_name);
        
        if !path.exists() {
            return Err(format!("Steering 文件不存在: {}", file_name));
        }
        
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        
        let metadata = fs::metadata(&path).ok();
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified_at = metadata
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y/%m/%d %H:%M:%S").to_string()
            });
        
        Ok(SteeringFile {
            file_name: file_name.to_string(),
            content,
            size,
            modified_at,
        })
    }

    /// 保存 steering 文件
    pub fn save(file_name: &str, content: &str) -> Result<(), String> {
        // 验证文件名，防止路径遍历
        Self::validate_file_name(file_name)?;

        let dir = Self::steering_dir().ok_or("无法获取用户目录")?;
        fs::create_dir_all(&dir).ok();

        let path = dir.join(file_name);
        fs::write(&path, content)
            .map_err(|e| format!("写入失败: {}", e))
    }

    /// 删除 steering 文件
    pub fn delete(file_name: &str) -> Result<(), String> {
        // 验证文件名，防止路径遍历
        Self::validate_file_name(file_name)?;

        let dir = Self::steering_dir().ok_or("无法获取用户目录")?;
        let path = dir.join(file_name);
        
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("删除失败: {}", e))?;
        }
        
        Ok(())
    }

    /// 创建新的 steering 文件
    pub fn create(file_name: &str, content: &str) -> Result<SteeringFile, String> {
        // 验证文件名，防止路径遍历
        Self::validate_file_name(file_name)?;

        let dir = Self::steering_dir().ok_or("无法获取用户目录")?;
        fs::create_dir_all(&dir).ok();

        let path = dir.join(file_name);
        
        if path.exists() {
            return Err(format!("文件已存在: {}", file_name));
        }
        
        fs::write(&path, content)
            .map_err(|e| format!("写入失败: {}", e))?;
        
        Self::load(file_name)
    }
}
