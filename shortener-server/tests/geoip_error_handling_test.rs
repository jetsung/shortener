/// GeoIP 错误处理集成测试
///
/// 测试需求:
/// - 8.5: WHEN GeoIP 数据库文件不存在 THEN 系统 SHALL 记录警告但不影响核心功能
/// - 12.4: WHEN 发生缓存错误 THEN 系统 SHALL 记录警告但不中断服务
///
/// 这个测试文件专门测试 GeoIP 模块的错误处理和降级机制
use shortener_server::geoip::{
    CachePolicy, GeoIp, GeoIpError, Ip2RegionGeoIp, NullGeoIp, create_geoip_with_fallback,
};
use std::path::PathBuf;

/// 辅助函数：获取测试数据库路径
fn get_test_db_path() -> Option<PathBuf> {
    let paths = vec![
        PathBuf::from("../data/ip2region.xdb"),
        PathBuf::from("./data/ip2region.xdb"),
        PathBuf::from("../../data/ip2region.xdb"),
        PathBuf::from("data/ip2region.xdb"),
    ];

    paths.into_iter().find(|p| p.exists())
}

#[tokio::test]
async fn test_database_not_found_graceful_degradation() {
    // 测试需求 8.5: 数据库文件不存在时应该记录警告但不影响核心功能

    // 使用不存在的数据库路径
    let geoip = create_geoip_with_fallback("/nonexistent/db.xdb", CachePolicy::NoCache);

    // 应该能够正常查询，返回空信息而不是崩溃
    let info = geoip.lookup_or_empty("8.8.8.8").await;
    assert!(
        info.is_empty(),
        "Should return empty info when database not found"
    );

    // 核心功能不受影响 - 可以继续处理其他请求
    let info2 = geoip.lookup_or_empty("1.1.1.1").await;
    assert!(
        info2.is_empty(),
        "Should continue to work after first failure"
    );
}

#[tokio::test]
async fn test_null_geoip_never_fails() {
    // 测试 NullGeoIp 永远不会失败
    let geoip = NullGeoIp::new();

    // 测试各种输入都不会失败
    let test_cases = vec![
        "",
        "invalid",
        "8.8.8.8",
        "999.999.999.999",
        "not.an.ip.address",
        "2001:4860:4860::8888",
    ];

    for ip in test_cases {
        let result = geoip.lookup(ip).await;
        assert!(result.is_ok(), "NullGeoIp should never fail for IP: {}", ip);

        let info = result.unwrap();
        assert!(info.is_empty(), "NullGeoIp should always return empty info");

        // 测试 lookup_or_empty
        let info2 = geoip.lookup_or_empty(ip).await;
        assert!(
            info2.is_empty(),
            "lookup_or_empty should also return empty info"
        );
    }
}

#[tokio::test]
async fn test_lookup_error_degradation() {
    // 测试查询失败时的降级处理
    let db_path = match get_test_db_path() {
        Some(path) => path,
        None => {
            println!("Skipping test: ip2region.xdb not found");
            return;
        }
    };

    let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

    // 测试空 IP 地址
    let result = geoip.lookup("").await;
    assert!(result.is_err(), "Empty IP should return error");

    // 但是 lookup_or_empty 应该返回空信息而不是错误
    let info = geoip.lookup_or_empty("").await;
    assert!(
        info.is_empty(),
        "lookup_or_empty should return empty info for invalid IP"
    );
}

#[tokio::test]
async fn test_error_does_not_affect_subsequent_requests() {
    // 测试错误不会影响后续请求
    let db_path = match get_test_db_path() {
        Some(path) => path,
        None => {
            println!("Skipping test: ip2region.xdb not found");
            return;
        }
    };

    let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

    // 第一个请求失败
    let info1 = geoip.lookup_or_empty("").await;
    assert!(info1.is_empty());

    // 第二个请求应该正常工作
    let info2 = geoip.lookup_or_empty("8.8.8.8").await;
    assert!(
        !info2.is_empty(),
        "Valid IP should return non-empty info after error"
    );

    // 第三个请求再次失败
    let info3 = geoip.lookup_or_empty("").await;
    assert!(info3.is_empty());

    // 第四个请求应该正常工作
    let info4 = geoip.lookup_or_empty("1.1.1.1").await;
    assert!(
        !info4.is_empty(),
        "Should continue to work after multiple errors"
    );
}

#[tokio::test]
async fn test_concurrent_requests_with_errors() {
    // 测试并发请求中的错误处理
    let db_path = match get_test_db_path() {
        Some(path) => path,
        None => {
            println!("Skipping test: ip2region.xdb not found");
            return;
        }
    };

    let geoip = std::sync::Arc::new(Ip2RegionGeoIp::with_default_cache(&db_path).unwrap());

    // 创建多个并发请求，包含有效和无效的 IP
    let mut handles = vec![];

    for i in 0..20 {
        let geoip_clone = geoip.clone();
        let handle = tokio::spawn(async move {
            let ip = match i % 4 {
                0 => "8.8.8.8", // Valid
                1 => "",        // Invalid (empty)
                2 => "1.1.1.1", // Valid
                3 => "invalid", // Invalid
                _ => unreachable!(),
            };

            // 使用 lookup_or_empty 确保不会失败
            let info = geoip_clone.lookup_or_empty(ip).await;
            (ip, info)
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    let mut valid_count = 0;
    let mut empty_count = 0;

    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent request should not panic");

        let (ip, info) = result.unwrap();
        if info.is_empty() {
            empty_count += 1;
        } else {
            valid_count += 1;
            println!("Valid lookup for {}: {:?}", ip, info);
        }
    }

    println!(
        "Valid lookups: {}, Empty lookups: {}",
        valid_count, empty_count
    );
    assert!(valid_count > 0, "Should have some valid lookups");
    assert!(
        empty_count > 0,
        "Should have some empty lookups from errors"
    );
}

#[tokio::test]
async fn test_fallback_creation_with_valid_and_invalid_paths() {
    // 测试使用有效和无效路径创建 GeoIP 实例

    // 无效路径应该返回 NullGeoIp
    let geoip1 = create_geoip_with_fallback("/nonexistent/db1.xdb", CachePolicy::NoCache);
    let info1 = geoip1.lookup_or_empty("8.8.8.8").await;
    assert!(info1.is_empty(), "Invalid path should result in NullGeoIp");

    // 如果有有效路径，应该返回真实的 GeoIP 实例
    if let Some(db_path) = get_test_db_path() {
        let geoip2 = create_geoip_with_fallback(&db_path, CachePolicy::VectorIndex);
        let info2 = geoip2.lookup_or_empty("8.8.8.8").await;
        assert!(
            !info2.is_empty(),
            "Valid path should result in working GeoIP"
        );
    }
}

#[tokio::test]
async fn test_error_types_are_descriptive() {
    // 测试错误类型包含有用的信息

    // DatabaseNotFound 错误
    let result = Ip2RegionGeoIp::new("/path/to/missing.xdb", CachePolicy::NoCache);
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("Database file not found"),
            "Error should mention database not found"
        );
        assert!(
            error_msg.contains("missing.xdb"),
            "Error should include the path"
        );
    }

    // InvalidIpAddress 错误
    if let Some(db_path) = get_test_db_path() {
        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();
        let result = geoip.lookup("").await;
        assert!(result.is_err());
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Invalid IP address"),
                "Error should mention invalid IP"
            );
        }
    }
}

#[tokio::test]
async fn test_geoip_trait_default_implementation() {
    // 测试 GeoIp trait 的默认 lookup_or_empty 实现

    if let Some(db_path) = get_test_db_path() {
        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

        // 测试有效 IP
        let info = geoip.lookup_or_empty("8.8.8.8").await;
        assert!(!info.is_empty(), "Valid IP should return non-empty info");

        // 测试无效 IP
        let info = geoip.lookup_or_empty("").await;
        assert!(
            info.is_empty(),
            "Invalid IP should return empty info via lookup_or_empty"
        );
    }
}

#[test]
fn test_geoip_error_variants() {
    // 测试所有 GeoIpError 变体
    use std::io;

    let errors = vec![
        GeoIpError::DatabaseNotFound("/test/path".to_string()),
        GeoIpError::InitializationError("init failed".to_string()),
        GeoIpError::LookupError("lookup failed".to_string()),
        GeoIpError::InvalidIpAddress("invalid".to_string()),
        GeoIpError::IoError(io::Error::new(io::ErrorKind::NotFound, "test")),
    ];

    for error in errors {
        let error_string = error.to_string();
        assert!(!error_string.is_empty(), "Error should have a message");
        println!("Error variant: {}", error_string);
    }
}

#[tokio::test]
async fn test_system_continues_without_geoip() {
    // 测试需求 8.5: 系统在 GeoIP 不可用时应该继续正常运行

    // 模拟 GeoIP 不可用的情况
    let geoip = NullGeoIp::new();

    // 模拟处理多个请求
    for i in 0..10 {
        let ip = format!("192.168.1.{}", i);
        let info = geoip.lookup_or_empty(&ip).await;

        // 即使 GeoIP 不可用，系统也应该继续处理
        assert!(info.is_empty(), "NullGeoIp should return empty info");

        // 在实际应用中，这里会继续处理其他业务逻辑
        // 例如记录访问历史，但不包含地理位置信息
    }

    println!("System successfully processed 10 requests without GeoIP data");
}

#[tokio::test]
async fn test_mixed_cache_policies_with_errors() {
    // 测试不同缓存策略下的错误处理

    let cache_policies = vec![
        CachePolicy::NoCache,
        CachePolicy::VectorIndex,
        CachePolicy::FullMemory,
    ];

    for policy in cache_policies {
        // 测试不存在的文件
        let geoip = create_geoip_with_fallback("/nonexistent/db.xdb", policy);
        let info = geoip.lookup_or_empty("8.8.8.8").await;
        assert!(
            info.is_empty(),
            "Should handle missing database with policy {:?}",
            policy
        );

        // 测试存在的文件（如果有）
        if let Some(db_path) = get_test_db_path() {
            let geoip = create_geoip_with_fallback(&db_path, policy);
            let info = geoip.lookup_or_empty("8.8.8.8").await;
            assert!(
                !info.is_empty(),
                "Should work with valid database and policy {:?}",
                policy
            );
        }
    }
}
