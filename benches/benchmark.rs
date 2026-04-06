use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::time::Duration;

// 内联密码哈希函数，避免导入问题
fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())
}

fn verify_password(hash: &str, password: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash).map_err(|e| e.to_string())
}

fn bench_password_hash(c: &mut Criterion) {
    let password = "test_password_123";
    
    c.bench_function("password_hash", |b| {
        b.iter(|| {
            hash_password(black_box(password)).unwrap();
        });
    });
    
    let hashed = hash_password(password).unwrap();
    c.bench_function("password_verify", |b| {
        b.iter(|| {
            verify_password(black_box(&hashed), black_box(password)).unwrap();
        });
    });
}

fn bench_string_operations(c: &mut Criterion) {
    let test_string = "This is a test string for benchmarking";
    
    c.bench_function("string_clone", |b| {
        b.iter(|| {
            let _ = black_box(test_string).to_string();
        });
    });
    
    c.bench_function("string_length", |b| {
        b.iter(|| {
            let _ = black_box(test_string).len();
        });
    });
}

fn bench_hash_map_operations(c: &mut Criterion) {
    let mut map = HashMap::new();
    for i in 0..1000 {
        map.insert(i, format!("value_{}", i));
    }
    
    c.bench_function("hash_map_get", |b| {
        b.iter(|| {
            let _ = map.get(&black_box(500));
        });
    });
    
    c.bench_function("hash_map_insert", |b| {
        b.iter(|| {
            let mut map = map.clone();
            map.insert(1001, "new_value".to_string());
        });
    });
}

fn bench_vector_operations(c: &mut Criterion) {
    let vec: Vec<i32> = (0..1000).collect();
    
    c.bench_function("vector_iterate", |b| {
        b.iter(|| {
            let sum: i32 = black_box(&vec).iter().sum();
            sum
        });
    });
    
    c.bench_function("vector_push", |b| {
        b.iter(|| {
            let mut vec = vec.clone();
            vec.push(1000);
        });
    });
}

criterion_group!(benches, 
    bench_password_hash, 
    bench_string_operations, 
    bench_hash_map_operations, 
    bench_vector_operations
);
criterion_main!(benches);
