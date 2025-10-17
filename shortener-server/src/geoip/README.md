# GeoIP 模块

此模块使用 ip2region 库提供 GeoIP 功能。

## 概述

GeoIP 模块允许你查找 IP 地址的地理信息。它使用 [ip2region](https://github.com/lionsoul2014/ip2region) 库，该库提供快速准确的 IP 地理位置数据。

## 特性

- **快速 IP 查找**：使用高效的二分搜索算法
- **多种缓存策略**：可选择无缓存、向量索引缓存或完全内存缓存
- **IPv4 和 IPv6 支持**：支持 IPv4 和 IPv6 地址（需要相应的数据库）
- **异步支持**：完全异步 API
- **线程安全**：可以安全地在多个线程之间共享

## 使用

### 基本用法

```rust
use shortener_server::geoip::{Ip2RegionGeoIp, GeoIp, CachePolicy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用默认缓存策略（VectorIndex）创建 GeoIP 实例
    let geoip = Ip2RegionGeoIp::with_default_cache("data/ip2region.xdb")?;
    
    // 查找 IP 地址
    let info = geoip.lookup("8.8.8.8").await?;
    
    println!("国家: {}", info.country);
    println!("地区: {}", info.region);
    println!("省份: {}", info.province);
    println!("城市: {}", info.city);
    println!("ISP: {}", info.isp);
    
    Ok(())
}
```

### 缓存策略

```rust
// 无缓存
let geoip = Ip2RegionGeoIp::new("data/ip2region.xdb", CachePolicy::None)?;

// 向量索引缓存（推荐）
let geoip = Ip2RegionGeoIp::new("data/ip2region.xdb", CachePolicy::VectorIndex)?;

// 完全内存缓存
let geoip = Ip2RegionGeoIp::new("data/ip2region.xdb", CachePolicy::Content)?;
```

## 数据库

从 [ip2region](https://github.com/lionsoul2014/ip2region) 下载 `ip2region.xdb` 数据库文件。

将其放在 `data/ip2region.xdb` 或在配置中指定路径。
