use prest::*;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

const START_TABLE_SIZE: u64 = 10_000; // 10x increase for stress testing
const ANALYTICS_TABLE_SIZE: u64 = 1_000; // 10x increase for analytics testing

const SPAWN_INTERVAL: u32 = 100;
const READS_PER_SPAWN: u64 = 100;
const UPDATES_PER_SPAWN: u64 = 10;
const SAVES_PER_SPAWN: u64 = 1;

#[derive(Clone, Storage, Serialize, Deserialize)]
struct Entry {
    pub id: u64,
    pub uuid: Uuid,
    #[unique]
    pub unique: String,
    pub optional: Option<String>,
    pub list: Vec<bool>,
    pub state: State,
}

// Analytics-like structure similar to RouteStat
#[derive(Clone, Storage, Serialize, Deserialize)]
struct AnalyticsEntry {
    #[pkey]
    pub path: String,
    pub method_stats: HashMap<String, (u64, f64)>, // Similar to RouteStat
    pub is_asset: bool,
    pub metadata: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum State {
    Simple,
    Complex { a: String, b: Option<f64> },
    Counter(u64),
}

impl Entry {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            uuid: Uuid::now_v7(),
            unique: id.to_string(),
            optional: Some("can be None instead".to_owned()),
            state: State::Complex {
                a: "Lorem ipsum dolor sit amet, consectetur adipiscing elit".to_owned(),
                b: Some(123.456),
            },
            list: vec![true, false],
        }
    }
}

impl AnalyticsEntry {
    pub fn new(id: u64) -> Self {
        let mut method_stats = HashMap::new();
        method_stats.insert("GET".to_string(), (fastrand::u64(100..10000), fastrand::f64() * 100.0));
        method_stats.insert("POST".to_string(), (fastrand::u64(10..1000), fastrand::f64() * 200.0));
        if fastrand::bool() {
            method_stats.insert("PUT".to_string(), (fastrand::u64(1..100), fastrand::f64() * 150.0));
        }
        if fastrand::bool() {
            method_stats.insert("DELETE".to_string(), (fastrand::u64(1..50), fastrand::f64() * 80.0));
        }

        Self {
            path: format!("/api/endpoint_{}", id),
            method_stats,
            is_asset: id % 5 == 0, // 20% are assets
            metadata: (0..fastrand::usize(1..10))
                .map(|i| format!("metadata_{}_{}", id, i))
                .collect(),
        }
    }
}

static ID_OFFSET: AtomicU64 = AtomicU64::new(1);
static ROWS_COUNT: AtomicU64 = AtomicU64::new(0);
static ANALYTICS_ROWS_COUNT: AtomicU64 = AtomicU64::new(0);

// Benchmark state
static CURRENT_BENCHMARK: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());

#[init]
async fn main() -> Result {
    route("/", get(dashboard))
        .route("/bench/:benchmark", post(run_benchmark))
        .route("/status", get(get_status))
        .run()
        .await?;

    OK
}

async fn dashboard() -> Markup {
    let current = CURRENT_BENCHMARK.read().unwrap().clone();
    let entry_count = ROWS_COUNT.load(Ordering::SeqCst);
    let analytics_count = ANALYTICS_ROWS_COUNT.load(Ordering::SeqCst);
    
    html! {
        (DOCTYPE)
        html {
            head {
                title { "Database Benchmark Suite" }
                style {
                    "
                    body { font-family: monospace; margin: 40px; }
                    .benchmark { margin: 20px 0; padding: 20px; border: 1px solid #ccc; }
                    button { padding: 10px 20px; margin: 5px; }
                    .running { background-color: #fff3cd; }
                    .results { background-color: #f8f9fa; padding: 10px; margin: 10px 0; }
                    "
                }
            }
            body {
                h1 { "Database Benchmark Suite" }
                
                div.results {
                    h3 { "Current Status" }
                    p { "Active Benchmark: " (if current.is_empty() { "None" } else { &current }) }
                    p { "Entry Table Size: " (entry_count) }
                    p { "Analytics Table Size: " (analytics_count) }
                }

                div.benchmark {
                    h3 { "Cleanup" }
                    p { "Clear all data" }
                    button onclick="runBenchmark('cleanup')" { "Run Cleanup" }
                }

                div.benchmark {
                    h3 { "1. Basic Setup" }
                    p { "Prepare database with test data" }
                    button onclick="runBenchmark('setup')" { "Run Setup" }
                }

                div.benchmark {
                    h3 { "2. Analytics Load Test" }
                    p { "Test analytics-like operations with HashMap serialization" }
                    button onclick="runBenchmark('analytics_load')" { "Run Analytics Load" }
                }

                div.benchmark {
                    h3 { "3. Full Table Scan Test" }
                    p { "Test full table scan performance like analytics page" }
                    button onclick="runBenchmark('full_scan')" { "Run Full Scan" }
                }

                div.benchmark {
                    h3 { "4. Memory Stress Test" }
                    p { "Test memory usage patterns with large data structures" }
                    button onclick="runBenchmark('memory_stress')" { "Run Memory Stress" }
                }

                div.benchmark {
                    h3 { "5. Concurrency Test" }
                    p { "Test multiple simultaneous analytics requests" }
                    button onclick="runBenchmark('concurrency')" { "Run Concurrency" }
                }

                div.benchmark {
                    h3 { "6. Background Interference Test" }
                    p { "Test analytics read while writes are happening" }
                    button onclick="runBenchmark('interference')" { "Run Interference" }
                }

                div.benchmark {
                    h3 { "7. Repeated Load Test" }
                    p { "Test repeated analytics loads for memory leaks" }
                    button onclick="runBenchmark('repeated')" { "Run Repeated" }
                }

                div.benchmark {
                    h3 { "8. Original Benchmark" }
                    p { "Run original high-throughput benchmark" }
                    button onclick="runBenchmark('original')" { "Run Original" }
                }

                div.benchmark {
                    h3 { "9. Single-Thread Bottleneck Test" }
                    p { "Test if single reader thread is the bottleneck" }
                    button onclick="runBenchmark('bottleneck')" { "Run Bottleneck" }
                }

                div.benchmark {
                    h3 { "10. Serialization Overhead Test" }
                    p { "Test bitcode serialization performance with large HashMaps" }
                    button onclick="runBenchmark('serialization')" { "Run Serialization" }
                }

                div.benchmark {
                    h3 { "11. Memory Allocator Test" }
                    p { "Test memory allocation patterns (MUSL vs glibc)" }
                    button onclick="runBenchmark('allocator')" { "Run Allocator" }
                }

                div.benchmark {
                    h3 { "12. Release Build Simulation" }
                    p { "Simulate release build behavior patterns" }
                    button onclick="runBenchmark('release_sim')" { "Run Release Sim" }
                }

                div.benchmark {
                    h3 { "13. Reader Thread Saturation" }
                    p { "Saturate the single database reader thread" }
                    button onclick="runBenchmark('reader_saturation')" { "Run Reader Sat" }
                }

                div.benchmark {
                    h3 { "14. Realistic Production Test" }
                    p { "Test with realistic data size (few dozen entries, couple methods)" }
                    button onclick="runBenchmark('realistic')" { "Run Realistic" }
                }

                div.benchmark {
                    h3 { "15. System Stress Test" }
                    p { "Test system-level factors (cache misses, memory pressure)" }
                    button onclick="runBenchmark('system_stress')" { "Run System Stress" }
                }

                div.benchmark {
                    h3 { "16. Real Data Analysis" }
                    p { "Analyze your actual analytics data structure" }
                    button onclick="runBenchmark('data_analysis')" { "Analyze Real Data" }
                }

                div.benchmark {
                    h3 { "17. Cleanup" }
                    p { "Clear all data" }
                    button onclick="runBenchmark('cleanup')" { "Run Cleanup" }
                }

                div.benchmark {
                    h3 { "18. Large Database Simulation" }
                    p { "Create 200MB+ database to test large database scan performance" }
                    button onclick="runBenchmark('large_db')" { "Run Large DB" }
                }

                div.benchmark {
                    h3 { "19. Scan Performance vs DB Size" }
                    p { "Test how scan performance degrades with database size" }
                    button onclick="runBenchmark('scan_scaling')" { "Run Scan Scaling" }
                }

                div.benchmark {
                    h3 { "20. Database Fragmentation Test" }
                    p { "Test performance with fragmented database (many deletes/updates)" }
                    button onclick="runBenchmark('fragmentation')" { "Run Fragmentation" }
                }

                div.benchmark {
                    h3 { "21. Production Scenario Test" }
                    p { "200MB database with many tables but small analytics table" }
                    button onclick="runBenchmark('production_scenario')" { "Run Production" }
                }

                div id="status" { }

                script {
                    "
                    async function runBenchmark(name) {
                        document.getElementById('status').innerHTML = 'Running ' + name + '...';
                        try {
                            const response = await fetch('/bench/' + name, { method: 'POST' });
                            const result = await response.text();
                            document.getElementById('status').innerHTML = result;
                            updateStatus();
                        } catch (e) {
                            document.getElementById('status').innerHTML = 'Error: ' + e.message;
                        }
                    }
                    
                    async function updateStatus() {
                        try {
                            const response = await fetch('/status');
                            const result = await response.text();
                            // Update counts if needed
                        } catch (e) {
                            console.log('Status update failed:', e);
                        }
                    }
                    
                    setInterval(updateStatus, 2000);
                    "
                }
            }
        }
    }
}

async fn run_benchmark(Path(benchmark): Path<String>) -> Result<impl IntoResponse> {
    *CURRENT_BENCHMARK.write().unwrap() = benchmark.clone();
    
    let start = Instant::now();
    let result = match benchmark.as_str() {
        "setup" => setup_benchmark().await,
        "analytics_load" => analytics_load_benchmark().await,
        "full_scan" => full_scan_benchmark().await,
        "memory_stress" => memory_stress_benchmark().await,
        "concurrency" => concurrency_benchmark().await,
        "interference" => interference_benchmark().await,
        "repeated" => repeated_benchmark().await,
        "original" => original_benchmark().await,
        "cleanup" => cleanup_benchmark().await,
        "bottleneck" => bottleneck_benchmark().await,
        "serialization" => serialization_benchmark().await,
        "allocator" => allocator_benchmark().await,
        "release_sim" => release_sim_benchmark().await,
        "reader_saturation" => reader_saturation_benchmark().await,
        "realistic" => realistic_benchmark().await,
        "system_stress" => system_stress_benchmark().await,
        "data_analysis" => data_analysis_benchmark().await,
        "large_db" => large_db_benchmark().await,
        "scan_scaling" => scan_scaling_benchmark().await,
        "fragmentation" => fragmentation_benchmark().await,
        "production_scenario" => production_scenario_benchmark().await,
        _ => Err(e!("Unknown benchmark: {}", benchmark)),
    };
    
    let duration = start.elapsed();
    *CURRENT_BENCHMARK.write().unwrap() = String::new();
    
    match result {
        Ok(message) => Ok(format!("✅ {} completed in {:?}\n{}", benchmark, duration, message)),
        Err(e) => Ok(format!("❌ {} failed in {:?}\nError: {}", benchmark, duration, e)),
    }
}

async fn get_status() -> impl IntoResponse {
    let entry_count = ROWS_COUNT.load(Ordering::SeqCst);
    let analytics_count = ANALYTICS_ROWS_COUNT.load(Ordering::SeqCst);
    format!("Entries: {}, Analytics: {}", entry_count, analytics_count)
}

// Benchmark implementations
async fn setup_benchmark() -> Result<String> {
    info!("Setting up benchmark data...");
    
    // Clear existing data
    DB.nuke().await?;
    ROWS_COUNT.store(0, Ordering::SeqCst);
    ANALYTICS_ROWS_COUNT.store(0, Ordering::SeqCst);
    
    // Create basic entries
    for i in 1..=START_TABLE_SIZE {
        if let Ok(_) = Entry::new(i).save().await {
            ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    // Create analytics entries
    for i in 1..=ANALYTICS_TABLE_SIZE {
        if let Ok(_) = AnalyticsEntry::new(i).save().await {
            ANALYTICS_ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    Ok(format!("Created {} entries and {} analytics entries", START_TABLE_SIZE, ANALYTICS_TABLE_SIZE))
}

async fn analytics_load_benchmark() -> Result<String> {
    let start = Instant::now();
    
    info!("Starting analytics load test...");
    
    // Test 1: Get all analytics entries (like RouteStat::get_all())
    let fetch_start = Instant::now();
    let analytics = AnalyticsEntry::get_all().await?;
    let fetch_time = fetch_start.elapsed();
    
    // Test 2: Process data like analytics page
    let process_start = Instant::now();
    let mut total_hits = 0;
    let mut method_count = 0;
    
    for entry in &analytics {
        method_count += entry.method_stats.len();
        for (_, (hits, _)) in &entry.method_stats {
            total_hits += hits;
        }
    }
    let process_time = process_start.elapsed();
    
    // Test 3: Simulate HTML generation overhead
    let render_start = Instant::now();
    let mut html_size = 0;
    for entry in &analytics {
        for (method, (_hits, _latency)) in &entry.method_stats {
            html_size += method.len() + entry.path.len() + 20; // Simulate markup
        }
    }
    let render_time = render_start.elapsed();
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Analytics Load Test:\n\
        - Entries: {}\n\
        - Methods: {}\n\
        - Total hits: {}\n\
        - Fetch time: {:?}\n\
        - Process time: {:?}\n\
        - Render time: {:?}\n\
        - Total time: {:?}\n\
        - HTML size: {} chars",
        analytics.len(), method_count, total_hits, 
        fetch_time, process_time, render_time, total_time, html_size
    ))
}

async fn full_scan_benchmark() -> Result<String> {
    let start = Instant::now();
    
    info!("Starting full scan test...");
    
    // Multiple full scans to test performance degradation
    let mut scan_times = vec![];
    
    for i in 1..=50 {
        let scan_start = Instant::now();
        let analytics = AnalyticsEntry::get_all().await?;
        let scan_time = scan_start.elapsed();
        scan_times.push(scan_time);
        
        info!("Scan {}: {:?} - {} entries", i, scan_time, analytics.len());
    }
    
    let total_time = start.elapsed();
    let avg_time = scan_times.iter().sum::<std::time::Duration>() / scan_times.len() as u32;
    
    Ok(format!(
        "Full Scan Test (50 iterations):\n\
        - Average time: {:?}\n\
        - First 5 times: {:?}\n\
        - Last 5 times: {:?}\n\
        - Total time: {:?}",
        avg_time, &scan_times[0..5.min(scan_times.len())], 
        &scan_times[scan_times.len().saturating_sub(5)..], total_time
    ))
}

async fn memory_stress_benchmark() -> Result<String> {
    let start = Instant::now();
    
    info!("Starting memory stress test...");
    
    // Create large analytics entries with big HashMaps
    let mut large_entries = vec![];
    for i in 1..=500 {
        let mut method_stats = HashMap::new();
        
        // Create many methods with large hit counts
        for j in 0..100 {
            method_stats.insert(
                format!("METHOD_{}_{}", i, j),
                (fastrand::u64(1000..100000), fastrand::f64() * 1000.0)
            );
        }
        
        let entry = AnalyticsEntry {
            path: format!("/stress/test/{}", i),
            method_stats,
            is_asset: false,
            metadata: (0..100).map(|k| format!("large_metadata_{}_{}", i, k)).collect(),
        };
        
        large_entries.push(entry);
    }
    
    // Save them
    let save_start = Instant::now();
    for entry in &large_entries {
        entry.save().await?;
    }
    let save_time = save_start.elapsed();
    
    // Read them back
    let read_start = Instant::now();
    let read_entries = AnalyticsEntry::get_all().await?;
    let read_time = read_start.elapsed();
    
    // Process them
    let process_start = Instant::now();
    let mut total_methods = 0;
    let mut total_metadata = 0;
    
    for entry in &read_entries {
        total_methods += entry.method_stats.len();
        total_metadata += entry.metadata.len();
    }
    let process_time = process_start.elapsed();
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Memory Stress Test:\n\
        - Large entries created: {}\n\
        - Total entries read: {}\n\
        - Total methods: {}\n\
        - Total metadata: {}\n\
        - Save time: {:?}\n\
        - Read time: {:?}\n\
        - Process time: {:?}\n\
        - Total time: {:?}",
        large_entries.len(), read_entries.len(), 
        total_methods, total_metadata,
        save_time, read_time, process_time, total_time
    ))
}

async fn concurrency_benchmark() -> Result<String> {
    let start = Instant::now();
    
    info!("Starting concurrency test...");
    
    // Test 1: Sequential analytics loads
    let sequential_start = Instant::now();
    for i in 1..=50 {
        let load_start = Instant::now();
        let analytics = AnalyticsEntry::get_all().await?;
        let load_time = load_start.elapsed();
        info!("Sequential load {}: {:?} - {} entries", i, load_time, analytics.len());
    }
    let sequential_time = sequential_start.elapsed();
    
    // Test 2: Concurrent analytics loads
    let concurrent_start = Instant::now();
    let mut handles = vec![];
    
    for i in 1..=50 {
        let handle = RT.spawn(async move {
            let load_start = Instant::now();
            let analytics = AnalyticsEntry::get_all().await?;
            let load_time = load_start.elapsed();
            info!("Concurrent load {}: {:?} - {} entries", i, load_time, analytics.len());
            Ok::<_, AnyError>(load_time)
        });
        handles.push(handle);
    }
    
    let join_results = join_all(handles).await;
    let results: Vec<std::time::Duration> = join_results
        .into_iter()
        .filter_map(|join_result| match join_result {
            Ok(Ok(duration)) => Some(duration),
            Ok(Err(e)) => { error!("Analytics load failed: {:?}", e); None }
            Err(e) => { error!("Task join failed: {}", e); None }
        })
        .collect();
    
    let concurrent_time = concurrent_start.elapsed();
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Concurrency Test:\n\
        - Sequential 50x loads: {:?}\n\
        - Concurrent 50x loads: {:?}\n\
        - Successful concurrent loads: {}\n\
        - Avg concurrent time: {:?}\n\
        - Speedup ratio: {:.2}x\n\
        - Total time: {:?}",
        sequential_time, concurrent_time, results.len(),
        if results.is_empty() { std::time::Duration::ZERO } else { results.iter().sum::<std::time::Duration>() / results.len() as u32 },
        sequential_time.as_secs_f64() / concurrent_time.as_secs_f64(),
        total_time
    ))
}

async fn interference_benchmark() -> Result<String> {
    let start = Instant::now();
    
    info!("Starting interference test...");
    
    // Test 1: Analytics read without background writes
    let clean_start = Instant::now();
    let analytics = AnalyticsEntry::get_all().await?;
    let clean_time = clean_start.elapsed();
    
    // Test 2: Start background writes
    let write_handle = RT.spawn(async {
        for i in 1..=500 {
            let entry = AnalyticsEntry::new(10000 + i);
            if let Err(e) = entry.save().await {
                error!("Background write failed: {}", e);
            }
            sleep(std::time::Duration::from_millis(1)).await;
        }
    });
    
    // Test 3: Analytics read WITH background writes
    sleep(std::time::Duration::from_millis(50)).await; // Let writes start
    
    let interfered_start = Instant::now();
    let analytics2 = AnalyticsEntry::get_all().await?;
    let interfered_time = interfered_start.elapsed();
    
    // Wait for background writes to complete
    write_handle.await;
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Interference Test:\n\
        - Clean read: {:?} ({} entries)\n\
        - Read during writes: {:?} ({} entries)\n\
        - Slowdown ratio: {:.2}x\n\
        - Total time: {:?}",
        clean_time, analytics.len(),
        interfered_time, analytics2.len(),
        interfered_time.as_secs_f64() / clean_time.as_secs_f64(),
        total_time
    ))
}

async fn repeated_benchmark() -> Result<String> {
    let start = Instant::now();
    
    info!("Starting repeated load test...");
    
    let mut times = vec![];
    let mut memory_usage = vec![];
    
    for i in 1..=200 {
        // Force garbage collection (if possible)
        // Note: Rust doesn't have explicit GC, but this might help with cleanup
        
        let iteration_start = Instant::now();
        
        // Load analytics data
        let analytics = AnalyticsEntry::get_all().await?;
        
        // Process the data (simulate analytics page processing)
        let mut total_hits = 0;
        let mut html_size = 0;
        
        for entry in analytics {
            for (method, (hits, latency)) in entry.method_stats {
                total_hits += hits;
                html_size += method.len() + entry.path.len() + 50; // Simulate HTML generation
                
                // Create some temporary strings to stress memory
                let _temp = format!("{} {} {} {:.3}ms", method, entry.path, hits, latency);
            }
        }
        
        let iteration_time = iteration_start.elapsed();
        times.push(iteration_time);
        
        // Simulate memory usage (rough estimate)
        memory_usage.push(html_size);
        
        info!("Iteration {}: {:?}, hits: {}, html_size: {}", 
            i, iteration_time, total_hits, html_size);
        
        // Small delay between iterations
        sleep(std::time::Duration::from_millis(50)).await;
    }
    
    let total_time = start.elapsed();
    let avg_time = times.iter().sum::<std::time::Duration>() / times.len() as u32;
    let min_time = times.iter().min().unwrap();
    let max_time = times.iter().max().unwrap();
    
    // Check for performance degradation
    let first_half_avg = times[..100].iter().sum::<std::time::Duration>() / 100;
    let second_half_avg = times[100..].iter().sum::<std::time::Duration>() / 100;
    
    Ok(format!(
        "Repeated Load Test (200 iterations):\n\
        - Average time: {:?}\n\
        - Min time: {:?}\n\
        - Max time: {:?}\n\
        - First half avg: {:?}\n\
        - Second half avg: {:?}\n\
        - Performance degradation: {:.2}x\n\
        - Total time: {:?}\n\
        - Memory range: {} - {} chars",
        avg_time, min_time, max_time,
        first_half_avg, second_half_avg,
        second_half_avg.as_secs_f64() / first_half_avg.as_secs_f64(),
        total_time,
        memory_usage.iter().min().unwrap(),
        memory_usage.iter().max().unwrap()
    ))
}

async fn original_benchmark() -> Result<String> {
    info!("Starting original benchmark...");
    
    // Run the original benchmark logic but don't run it indefinitely
    let start = Instant::now();
    
    // Run for 10 seconds
    let mut operations = 0;
    while start.elapsed().as_secs() < 10 {
        // Simulate the original operations
        for _ in 0..10 {
            get_random().await;
            operations += 1;
        }
        for _ in 0..2 {
            update().await;
            operations += 1;
        }
        save().await;
        operations += 1;
        
        sleep(std::time::Duration::from_millis(SPAWN_INTERVAL as u64)).await;
    }
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Original Benchmark (10 seconds):\n\
        - Operations: {}\n\
        - Rate: {:.1} ops/sec\n\
        - Total time: {:?}",
        operations, operations as f64 / total_time.as_secs_f64(), total_time
    ))
}

async fn cleanup_benchmark() -> Result<String> {
    info!("Cleaning up all data...");
    
    let start = Instant::now();
    DB.nuke().await?;
    let cleanup_time = start.elapsed();
    
    ROWS_COUNT.store(0, Ordering::SeqCst);
    ANALYTICS_ROWS_COUNT.store(0, Ordering::SeqCst);
    
    Ok(format!("Cleanup completed in {:?}", cleanup_time))
}

// Helper functions from original benchmark
async fn get_random() {
    let id = random_id();
    match Entry::get_by_pkey(id).await {
        Ok(Some(e)) => assert!(e.id == id),
        Ok(None) => error!("failed to read one: entry not found (id = {id})"),
        Err(e) => error!("failed to get_by_pkey: {e}"),
    };
}

async fn save() {
    let id = ID_OFFSET.fetch_add(1, Ordering::SeqCst);
    match Entry::new(id).save().await {
        Ok(_) => {
            ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        Err(e) => error!("failed to save: {e}"),
    }
}

async fn update() {
    let id = random_id();

    let Some(mut entry) = Entry::get_by_pkey(id)
        .await
        .expect("get_by_pkey with existing PK value should be fine")
    else {
        return; // Entry might not exist
    };

    let new_state = match entry.state {
        State::Simple => unreachable!("never assigned"),
        State::Complex { .. } => State::Counter(1),
        State::Counter(c) => State::Counter(c + 1),
    };

    if let Err(e) = entry.update_state(new_state).await {
        error!("failed to update: {e}");
    }
}

fn random_id() -> u64 {
    let count = ROWS_COUNT.load(Ordering::SeqCst);
    if count == 0 { 1 } else { fastrand::u64(1..=count) }
}

// NEW DIAGNOSTIC BENCHMARKS TO IDENTIFY ROOT CAUSE

async fn bottleneck_benchmark() -> Result<String> {
    info!("Testing single-thread bottleneck theory...");
    
    let start = Instant::now();
    
    // Test 1: Single large analytics read (simulates current problem)
    let large_read_start = Instant::now();
    let analytics = AnalyticsEntry::get_all().await?;
    let large_read_time = large_read_start.elapsed();
    
    // Test 2: Many small reads (should be fast if no bottleneck)
    let small_reads_start = Instant::now();
    let mut small_read_times = vec![];
    
    for i in 1..=50 {
        let read_start = Instant::now();
        let _entry = Entry::get_by_pkey(i).await?;
        small_read_times.push(read_start.elapsed());
    }
    let small_reads_time = small_reads_start.elapsed();
    
    // Test 3: Mixed load - analytics during small reads
    let mixed_start = Instant::now();
    
    // Start background small reads
    let bg_reads_handle = RT.spawn(async {
        let mut times = vec![];
        for i in 1..=100 {
            let read_start = Instant::now();
            let _entry = Entry::get_by_pkey(i % 100 + 1).await.unwrap_or_default();
            times.push(read_start.elapsed());
            sleep(std::time::Duration::from_millis(5)).await;
        }
        times
    });
    
    // Analytics read during background load
    sleep(std::time::Duration::from_millis(25)).await; // Let background start
    let analytics_during_load_start = Instant::now();
    let _analytics2 = AnalyticsEntry::get_all().await?;
    let analytics_during_load_time = analytics_during_load_start.elapsed();
    
    let bg_read_times = bg_reads_handle.await.unwrap();
    let _mixed_time = mixed_start.elapsed();
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Single-Thread Bottleneck Test:\n\
        - Large analytics read: {:?} ({} entries)\n\
        - 50 small reads: {:?} (avg: {:?})\n\
        - Analytics during load: {:?}\n\
        - Background reads affected: {:.2}x slowdown\n\
        - Total time: {:?}\n\
        - DIAGNOSIS: {}",
        large_read_time, analytics.len(),
        small_reads_time, small_read_times.iter().sum::<std::time::Duration>() / small_read_times.len() as u32,
        analytics_during_load_time,
        if bg_read_times.len() > 10 {
            let first_10_avg = bg_read_times[..10].iter().sum::<std::time::Duration>() / 10;
            let last_10_avg = bg_read_times[bg_read_times.len()-10..].iter().sum::<std::time::Duration>() / 10;
            last_10_avg.as_secs_f64() / first_10_avg.as_secs_f64()
        } else { 1.0 },
        total_time,
        if analytics_during_load_time > large_read_time * 2 {
            "BOTTLENECK CONFIRMED - Single reader thread is saturated"
        } else {
            "No significant bottleneck detected"
        }
    ))
}

async fn serialization_benchmark() -> Result<String> {
    info!("Testing serialization overhead...");
    
    let start = Instant::now();
    
    // Test 1: Create entries with different HashMap sizes
    let mut entries_small = vec![];
    let mut entries_large = vec![];
    
    // Small HashMaps (like typical usage)
    for i in 1..=100 {
        let mut method_stats = HashMap::new();
        method_stats.insert("GET".to_string(), (fastrand::u64(100..1000), fastrand::f64() * 50.0));
        method_stats.insert("POST".to_string(), (fastrand::u64(10..100), fastrand::f64() * 100.0));
        
        entries_small.push(AnalyticsEntry {
            path: format!("/small/{}", i),
            method_stats,
            is_asset: false,
            metadata: vec![format!("meta_{}", i)],
        });
    }
    
    // Large HashMaps (stress test)
    for i in 1..=50 {
        let mut method_stats = HashMap::new();
        for j in 0..200 { // 200 methods per entry!
            method_stats.insert(
                format!("METHOD_{}_{}", i, j),
                (fastrand::u64(1..10000), fastrand::f64() * 500.0)
            );
        }
        
        entries_large.push(AnalyticsEntry {
            path: format!("/large/{}", i),
            method_stats,
            is_asset: false,
            metadata: (0..50).map(|k| format!("large_meta_{}_{}", i, k)).collect(),
        });
    }
    
    // Test 2: Serialization timing (using JSON as proxy for bitcode)
    let serialize_start = Instant::now();
    let mut serialize_times_small = vec![];
    let mut serialize_times_large = vec![];
    
    for entry in &entries_small {
        let ser_start = Instant::now();
        let _serialized = to_json_vec(entry).unwrap();
        serialize_times_small.push(ser_start.elapsed());
    }
    
    for entry in &entries_large {
        let ser_start = Instant::now();
        let _serialized = to_json_vec(entry).unwrap();
        serialize_times_large.push(ser_start.elapsed());
    }
    let serialize_time = serialize_start.elapsed();
    
    // Test 3: Database save timing
    let save_start = Instant::now();
    for entry in &entries_small {
        entry.save().await?;
    }
    for entry in &entries_large {
        entry.save().await?;
    }
    let save_time = save_start.elapsed();
    
    // Test 4: Full read back (this is what analytics page does)
    let read_start = Instant::now();
    let all_entries = AnalyticsEntry::get_all().await?;
    let read_time = read_start.elapsed();
    
    let total_time = start.elapsed();
    
    let avg_small_ser = serialize_times_small.iter().sum::<std::time::Duration>() / serialize_times_small.len() as u32;
    let avg_large_ser = serialize_times_large.iter().sum::<std::time::Duration>() / serialize_times_large.len() as u32;
    
    Ok(format!(
        "Serialization Overhead Test:\n\
        - Small HashMap entries: {}\n\
        - Large HashMap entries: {}\n\
        - Avg small serialization: {:?}\n\
        - Avg large serialization: {:?}\n\
        - Serialization slowdown: {:.2}x\n\
        - Save time: {:?}\n\
        - Read time: {:?} ({} entries)\n\
        - Total time: {:?}\n\
        - DIAGNOSIS: {}",
        entries_small.len(), entries_large.len(),
        avg_small_ser, avg_large_ser,
        avg_large_ser.as_secs_f64() / avg_small_ser.as_secs_f64(),
        save_time, read_time, all_entries.len(),
        total_time,
        if avg_large_ser > avg_small_ser * 10 {
            "SERIALIZATION OVERHEAD CONFIRMED - Large HashMaps are expensive"
        } else {
            "Serialization overhead is acceptable"
        }
    ))
}

async fn allocator_benchmark() -> Result<String> {
    info!("Testing memory allocator patterns...");
    
    let start = Instant::now();
    
    // Test 1: Many small allocations (typical of HashMap serialization)
    let small_alloc_start = Instant::now();
    let mut small_vecs = vec![];
    
    for i in 0..10000 {
        let mut data = HashMap::new();
        for j in 0..10 {
            data.insert(format!("key_{}_{}", i, j), (fastrand::u64(1..1000), fastrand::f64()));
        }
        let serialized = into_bitcode(&data).unwrap();
        small_vecs.push(serialized);
    }
    let small_alloc_time = small_alloc_start.elapsed();
    
    // Test 2: Large contiguous allocations
    let large_alloc_start = Instant::now();
    let mut large_vecs = vec![];
    
    for i in 0..100 {
        let mut data = HashMap::new();
        for j in 0..1000 {
            data.insert(format!("large_key_{}_{}", i, j), (fastrand::u64(1..10000), fastrand::f64() * 1000.0));
        }
        let serialized = into_bitcode(&data).unwrap();
        large_vecs.push(serialized);
    }
    let large_alloc_time = large_alloc_start.elapsed();
    
    // Test 3: Fragmentation test - interleaved alloc/dealloc
    let frag_start = Instant::now();
    let mut temp_storage = vec![];
    
    for cycle in 0..1000 {
        // Allocate
        for i in 0..50 {
            let mut data = HashMap::new();
            for j in 0..20 {
                data.insert(format!("frag_{}_{}", cycle, j), (fastrand::u64(1..5000), fastrand::f64()));
            }
            temp_storage.push(into_bitcode(&data).unwrap());
        }
        
        // Deallocate half
        temp_storage.truncate(temp_storage.len() / 2);
    }
    let frag_time = frag_start.elapsed();
    
    // Test 4: Memory pressure simulation
    let pressure_start = Instant::now();
    let mut pressure_data = vec![];
    
    // Allocate progressively larger amounts
    for i in 1..=20 {
        let size = i * 1000;
        let mut batch = vec![];
        for j in 0..size {
            let mut data = HashMap::new();
            data.insert("method".to_string(), (j as u64, j as f64));
            batch.push(into_bitcode(&data).unwrap());
        }
        pressure_data.push(batch);
    }
    let pressure_time = pressure_start.elapsed();
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Memory Allocator Test:\n\
        - Small allocations: {:?} (10K x 10 keys)\n\
        - Large allocations: {:?} (100 x 1K keys)\n\
        - Fragmentation test: {:?}\n\
        - Memory pressure: {:?}\n\
        - Total time: {:?}\n\
        - Small allocation rate: {:.0} allocs/sec\n\
        - Large allocation rate: {:.0} allocs/sec\n\
        - DIAGNOSIS: {}",
        small_alloc_time, large_alloc_time, frag_time, pressure_time, total_time,
        10000.0 / small_alloc_time.as_secs_f64(),
        100.0 / large_alloc_time.as_secs_f64(),
        if large_alloc_time > small_alloc_time * 20 {
            "ALLOCATOR ISSUES - Large allocations disproportionately slow (possible MUSL impact)"
        } else {
            "Memory allocator performance is normal"
        }
    ))
}

async fn release_sim_benchmark() -> Result<String> {
    info!("Simulating release build behavior patterns...");
    
    let start = Instant::now();
    
    // Test 1: Optimize for throughput (release build behavior)
    let throughput_start = Instant::now();
    let mut results = vec![];
    
    // Batch operations more aggressively
    for _batch in 0..10 {
        let batch_start = Instant::now();
        
        // Simulate intense analytics processing
        let analytics = AnalyticsEntry::get_all().await?;
        
        // Process with tight loops (release optimizations)
        let mut total_hits = 0u64;
        let mut _total_latency = 0f64;
        let mut method_count = 0;
        
        for entry in analytics {
            for (method, (hits, latency)) in entry.method_stats {
                total_hits = total_hits.wrapping_add(hits);
                _total_latency += latency;
                method_count += 1;
                
                // Simulate string operations (common in analytics)
                let _formatted = format!("{} {} {:.3}ms", method, entry.path, latency);
            }
        }
        
        let batch_time = batch_start.elapsed();
        results.push((batch_time, total_hits, method_count));
    }
    let throughput_time = throughput_start.elapsed();
    
    // Test 2: Memory layout optimization simulation
    let layout_start = Instant::now();
    
    // Create structures that might behave differently in release
    #[derive(Serialize, Deserialize)]
    struct OptimizedAnalytics {
        path: String,
        methods: Vec<(String, u64, f64)>, // Flattened instead of HashMap
        is_asset: bool,
    }
    
    let analytics = AnalyticsEntry::get_all().await?;
    let mut optimized = vec![];
    
    for entry in analytics {
        let methods: Vec<(String, u64, f64)> = entry.method_stats
            .into_iter()
            .map(|(k, (hits, latency))| (k, hits, latency))
            .collect();
        
        optimized.push(OptimizedAnalytics {
            path: entry.path,
            methods,
            is_asset: entry.is_asset,
        });
    }
    let layout_time = layout_start.elapsed();
    
    // Test 3: Aggressive caching simulation
    let cache_start = Instant::now();
    
    // Simulate what a production cache would do
    static CACHE_COUNTER: AtomicU64 = AtomicU64::new(0);
    
    for i in 0..100 {
        let _cache_key = format!("analytics_cache_{}", i % 5); // 5 cache buckets
        
        // Simulate cache miss/hit pattern
        if CACHE_COUNTER.fetch_add(1, Ordering::SeqCst) % 10 == 0 {
            // Cache miss - full load
            let _analytics = AnalyticsEntry::get_all().await?;
        } else {
            // Cache hit - minimal work
            sleep(std::time::Duration::from_micros(100)).await;
        }
    }
    let cache_time = cache_start.elapsed();
    
    let total_time = start.elapsed();
    
    let avg_batch_time = results.iter().map(|(t, _, _)| *t).sum::<std::time::Duration>() / results.len() as u32;
    let total_methods: usize = results.iter().map(|(_, _, m)| *m).sum();
    
    Ok(format!(
        "Release Build Simulation:\n\
        - Throughput test: {:?} (10 batches)\n\
        - Avg batch time: {:?}\n\
        - Total methods processed: {}\n\
        - Layout optimization: {:?}\n\
        - Cache simulation: {:?}\n\
        - Total time: {:?}\n\
        - Methods/sec: {:.0}\n\
        - DIAGNOSIS: {}",
        throughput_time, avg_batch_time, total_methods,
        layout_time, cache_time, total_time,
        total_methods as f64 / total_time.as_secs_f64(),
        if avg_batch_time > std::time::Duration::from_millis(500) {
            "RELEASE BUILD ISSUES - Batch processing too slow for production"
        } else {
            "Release build simulation shows acceptable performance"
        }
    ))
}

async fn reader_saturation_benchmark() -> Result<String> {
    info!("Testing database reader thread saturation...");
    
    let start = Instant::now();
    
    // Test 1: Saturate with many concurrent reads
    let saturation_start = Instant::now();
    let mut handles = vec![];
    
    // Spawn many tasks that will all hit the single reader thread
    for batch in 0..5 {
        for i in 1..=20 { // 100 total tasks
            let handle = RT.spawn(async move {
                let task_start = Instant::now();
                
                // Different types of reads to stress the reader thread
                match i % 4 {
                    0 => {
                        // Full analytics scan (expensive)
                        let _analytics = AnalyticsEntry::get_all().await?;
                    }
                    1 => {
                        // Entry lookup (fast)
                        let _entry = Entry::get_by_pkey(fastrand::u64(1..1000)).await?;
                    }
                    2 => {
                        // Multiple entry lookups
                        for j in 1..=10 {
                            let _entry = Entry::get_by_pkey(j).await?;
                        }
                    }
                    3 => {
                        // Full entry scan
                        let _entries = Entry::get_all().await?;
                    }
                    _ => unreachable!()
                }
                
                Ok::<_, AnyError>((batch, i, task_start.elapsed()))
            });
            handles.push(handle);
        }
        
        // Small delay between batches to observe queuing
        sleep(std::time::Duration::from_millis(10)).await;
    }
    
    // Wait for all tasks
    let results = join_all(handles).await;
    let saturation_time = saturation_start.elapsed();
    
    // Analyze results
    let successful_tasks: Vec<(u32, u32, std::time::Duration)> = results
        .into_iter()
        .filter_map(|join_result| match join_result {
            Ok(Ok(task_result)) => Some(task_result),
            Ok(Err(e)) => { error!("Task failed: {:?}", e); None }
            Err(e) => { error!("Join failed: {:?}", e); None }
        })
        .collect();
    
    // Test 2: Measure queue depth impact
    let queue_test_start = Instant::now();
    let mut queue_times = vec![];
    
    // Create artificial queue by rapid-fire requests
    for i in 0..50 {
        let request_start = Instant::now();
        let _entry = Entry::get_by_pkey((i % 100) + 1).await?;
        queue_times.push(request_start.elapsed());
        
        if i % 10 == 0 {
            // Inject expensive operation to create queue backup
            let _analytics = AnalyticsEntry::get_all().await?;
        }
    }
    let queue_test_time = queue_test_start.elapsed();
    
    let total_time = start.elapsed();
    
    // Calculate statistics
    let task_times: Vec<std::time::Duration> = successful_tasks.iter().map(|(_, _, t)| *t).collect();
    let avg_task_time = if task_times.is_empty() { 
        std::time::Duration::ZERO 
    } else { 
        task_times.iter().sum::<std::time::Duration>() / task_times.len() as u32 
    };
    let max_task_time = task_times.iter().max().copied().unwrap_or(std::time::Duration::ZERO);
    let min_task_time = task_times.iter().min().copied().unwrap_or(std::time::Duration::ZERO);
    
    let avg_queue_time = queue_times.iter().sum::<std::time::Duration>() / queue_times.len() as u32;
    
    Ok(format!(
        "Reader Thread Saturation Test:\n\
        - Concurrent tasks launched: 100\n\
        - Successful tasks: {}\n\
        - Saturation time: {:?}\n\
        - Avg task time: {:?}\n\
        - Min task time: {:?}\n\
        - Max task time: {:?}\n\
        - Queue test time: {:?}\n\
        - Avg queue time: {:?}\n\
        - Total time: {:?}\n\
        - DIAGNOSIS: {}",
        successful_tasks.len(), saturation_time,
        avg_task_time, min_task_time, max_task_time,
        queue_test_time, avg_queue_time, total_time,
        if max_task_time > avg_task_time * 10 {
            "READER SATURATION CONFIRMED - Single thread cannot handle concurrent load"
        } else if successful_tasks.len() < 80 {
            "READER THREAD FAILURE - Many tasks failed to complete"
        } else {
            "Reader thread handling concurrent load acceptably"
        }
    ))
}

async fn realistic_benchmark() -> Result<String> {
    info!("Testing with realistic data size (few dozen entries, couple methods)");
    
    let start = Instant::now();
    
    // Clear and create realistic production-like data
    DB.nuke().await?;
    
    // Typical routes in a small application
    let realistic_routes = vec![
        ("/", vec!["GET"]),
        ("/api/health", vec!["GET"]),
        ("/api/users", vec!["GET", "POST"]),
        ("/api/users/:id", vec!["GET", "PUT", "DELETE"]),
        ("/api/login", vec!["POST"]),
        ("/api/logout", vec!["POST"]),
        ("/admin", vec!["GET"]),
        ("/admin/analytics", vec!["GET"]),
        ("/admin/schedule", vec!["GET"]),
        ("/admin/db", vec!["GET"]),
        ("/static/style.css", vec!["GET"]),
        ("/static/script.js", vec!["GET"]),
        ("/favicon.ico", vec!["GET"]),
        ("/sw.js", vec!["GET"]),
        ("/manifest.json", vec!["GET"]),
    ];
    
    let setup_start = Instant::now();
    for (path, methods) in &realistic_routes {
        let mut method_stats = HashMap::new();
        for method in methods {
            let hits = match *method {
                "GET" => fastrand::u64(10..500), // GET requests are common
                "POST" => fastrand::u64(1..50),
                "PUT" | "DELETE" => fastrand::u64(1..20),
                _ => fastrand::u64(1..10),
            };
            let latency = match path.contains("static") || path.contains("favicon") {
                true => fastrand::f64() * 5.0, // Assets are fast
                false => fastrand::f64() * 100.0 + 10.0, // API calls vary
            };
            method_stats.insert(method.to_string(), (hits, latency));
        }
        
        let entry = AnalyticsEntry {
            path: path.to_string(),
            method_stats,
            is_asset: path.contains("static") || path.contains("favicon") || path.contains(".css") || path.contains(".js"),
            metadata: vec!["created_from_realistic_test".to_string()],
        };
        entry.save().await?;
    }
    let setup_time = setup_start.elapsed();
    
    // Test multiple analytics page loads (like real usage)
    let mut load_times = vec![];
    for i in 1..=10 {
        let load_start = Instant::now();
        let analytics = AnalyticsEntry::get_all().await?;
        let load_time = load_start.elapsed();
        load_times.push(load_time);
        
        // Simulate processing the data like the real analytics page
        let mut total_hits = 0;
        let mut method_count = 0;
        for entry in analytics {
            for (_, (hits, _)) in entry.method_stats {
                total_hits += hits;
                method_count += 1;
            }
        }
        
        info!("Load {}: {:?} - {} methods, {} total hits", i, load_time, method_count, total_hits);
    }
    
    let total_time = start.elapsed();
    let avg_load_time = load_times.iter().sum::<std::time::Duration>() / load_times.len() as u32;
    let max_load_time = load_times.iter().max().unwrap();
    let min_load_time = load_times.iter().min().unwrap();
    
    Ok(format!(
        "Realistic Production Test (15 routes, ~30 methods total):\n\
        - Setup time: {:?}\n\
        - Avg load time: {:?}\n\
        - Min load time: {:?}\n\
        - Max load time: {:?}\n\
        - Load times: {:?}\n\
        - Total time: {:?}\n\
        - DIAGNOSIS: {}",
        setup_time, avg_load_time, min_load_time, max_load_time, load_times, total_time,
        if avg_load_time > std::time::Duration::from_millis(100) {
            "SLOW - Even realistic data is taking too long!"
        } else {
            "FAST - Realistic data loads quickly, issue might be elsewhere"
        }
    ))
}

async fn system_stress_benchmark() -> Result<String> {
    info!("Testing system-level factors (cache misses, memory pressure, concurrent load)");
    
    let start = Instant::now();
    
    // Setup realistic data first
    DB.nuke().await?;
    for i in 1..=20 {
        let mut method_stats = HashMap::new();
        method_stats.insert("GET".to_string(), (fastrand::u64(10..100), fastrand::f64() * 50.0));
        method_stats.insert("POST".to_string(), (fastrand::u64(1..20), fastrand::f64() * 100.0));
        
        let entry = AnalyticsEntry {
            path: format!("/stress/{}", i),
            method_stats,
            is_asset: i % 5 == 0,
            metadata: vec![format!("stress_meta_{}", i)],
        };
        entry.save().await?;
    }
    
    // Test 1: Cache thrashing - create lots of unrelated data
    let cache_stress_start = Instant::now();
    info!("Creating cache pressure with large unrelated data...");
    
    // Fill database cache with unrelated large entries
    for i in 1..=1000 {
        let large_entry = Entry::new(i);
        let _ = large_entry.save().await; // Ignore errors
    }
    
    // Now try analytics read (should have cache misses)
    let analytics_with_cache_pressure = AnalyticsEntry::get_all().await?;
    let cache_stress_time = cache_stress_start.elapsed();
    
    // Test 2: Memory pressure simulation
    let memory_pressure_start = Instant::now();
    info!("Creating memory pressure...");
    
    // Allocate large amounts of memory to stress the allocator
    let mut memory_hogs: Vec<Vec<u8>> = vec![];
    for i in 0..100 {
        let size = 1024 * 1024; // 1MB chunks
        memory_hogs.push(vec![i as u8; size]);
    }
    
    // Analytics read under memory pressure
    let analytics_with_memory_pressure = AnalyticsEntry::get_all().await?;
    let memory_pressure_time = memory_pressure_start.elapsed();
    
    // Test 3: Concurrent load simulation (single reader thread stress)
    let concurrent_start = Instant::now();
    info!("Testing concurrent database load...");
    
    let mut handles = vec![];
    
    // Background: continuous small reads
    for i in 0..5 {
        let handle = RT.spawn(async move {
            let mut times = vec![];
            for j in 0..20 {
                let read_start = Instant::now();
                let _entry = Entry::get_by_pkey((i * 20 + j + 1) as u64).await.unwrap_or_default();
                times.push(read_start.elapsed());
                sleep(std::time::Duration::from_millis(10)).await;
            }
            times
        });
        handles.push(handle);
    }
    
    // Foreground: analytics reads during concurrent load
    sleep(std::time::Duration::from_millis(50)).await; // Let background start
    
    let mut analytics_times = vec![];
    for _ in 0..5 {
        let analytics_start = Instant::now();
        let _analytics = AnalyticsEntry::get_all().await?;
        analytics_times.push(analytics_start.elapsed());
        sleep(std::time::Duration::from_millis(20)).await;
    }
    
    // Wait for background tasks
    let background_results = join_all(handles).await;
    let concurrent_time = concurrent_start.elapsed();
    
    // Clean up memory
    drop(memory_hogs);
    
    let total_time = start.elapsed();
    
    let avg_analytics_time = analytics_times.iter().sum::<std::time::Duration>() / analytics_times.len() as u32;
    let background_task_count: usize = background_results.iter()
        .map(|r| r.as_ref().map(|times| times.len()).unwrap_or(0))
        .sum();
    
    Ok(format!(
        "System Stress Test:\n\
        - Cache stress time: {:?} ({} entries)\n\
        - Memory pressure time: {:?} ({} entries)\n\
        - Concurrent test time: {:?}\n\
        - Avg analytics under load: {:?}\n\
        - Background tasks completed: {}\n\
        - Analytics times under load: {:?}\n\
        - Total time: {:?}\n\
        - DIAGNOSIS: {}",
        cache_stress_time, analytics_with_cache_pressure.len(),
        memory_pressure_time, analytics_with_memory_pressure.len(),
        concurrent_time, avg_analytics_time, background_task_count,
        analytics_times, total_time,
        if avg_analytics_time > std::time::Duration::from_millis(500) {
            "SYSTEM BOTTLENECK - Performance degrades under system stress"
        } else {
            "SYSTEM OK - Performance remains good under stress"
        }
    ))
}

async fn data_analysis_benchmark() -> Result<String> {
    info!("Analyzing actual analytics data patterns and potential bottlenecks");
    
    let start = Instant::now();
    
    // Check if there's existing analytics data
    let existing_analytics = AnalyticsEntry::get_all().await?;
    
    if !existing_analytics.is_empty() {
        info!("Found {} existing analytics entries, analyzing...", existing_analytics.len());
        
        // Analyze the existing data
        let mut total_methods = 0;
        let mut total_string_length = 0;
        let mut method_counts = HashMap::new();
        let mut path_lengths = vec![];
        let mut largest_entry_methods = 0;
        let mut method_distribution = vec![];
        
        for entry in &existing_analytics {
            let method_count = entry.method_stats.len();
            total_methods += method_count;
            method_distribution.push(method_count);
            
            if method_count > largest_entry_methods {
                largest_entry_methods = method_count;
            }
            
            path_lengths.push(entry.path.len());
            total_string_length += entry.path.len();
            
            for (method, (hits, _)) in &entry.method_stats {
                total_string_length += method.len();
                *method_counts.entry(method.clone()).or_insert(0) += 1;
                
                // Check for unusually high hit counts that might indicate data accumulation
                if *hits > 10000 {
                    info!("Found high hit count: {} hits for {} {}", hits, method, entry.path);
                }
            }
        }
        
        // Sort method distribution to find outliers
        method_distribution.sort();
        let median_methods = method_distribution[method_distribution.len() / 2];
        let max_methods = method_distribution.last().copied().unwrap_or(0);
        
        // Test read performance
        let read_start = Instant::now();
        let _analytics = AnalyticsEntry::get_all().await?;
        let read_time = read_start.elapsed();
        
        let total_time = start.elapsed();
        
        Ok(format!(
            "Real Data Analysis:\n\
            - Total entries: {}\n\
            - Total methods: {}\n\
            - Avg methods per entry: {:.1}\n\
            - Median methods per entry: {}\n\
            - Max methods in single entry: {}\n\
            - Method distribution: {:?}\n\
            - Total string data: {} chars\n\
            - Most common methods: {:?}\n\
            - Read time: {:?}\n\
            - Total analysis time: {:?}\n\
            - DIAGNOSIS: {}",
            existing_analytics.len(),
            total_methods,
            total_methods as f64 / existing_analytics.len() as f64,
            median_methods,
            max_methods,
            &method_distribution[..5.min(method_distribution.len())],
            total_string_length,
            method_counts.iter().take(5).collect::<Vec<_>>(),
            read_time,
            total_time,
            if existing_analytics.len() > 100 {
                "POTENTIAL ISSUE - More analytics entries than expected! Data accumulation over time."
            } else if max_methods > 10 {
                "POTENTIAL ISSUE - Some entries have many methods! Could indicate route parameter issues."
            } else if read_time > std::time::Duration::from_millis(100) {
                "CONFIRMED ISSUE - Even current data is slow to read!"
            } else {
                "DATA LOOKS NORMAL - Issue might be system-level or concurrent load"
            }
        ))
    } else {
        // No existing data, create minimal test to understand baseline
        info!("No existing analytics data found, creating minimal test...");
        
        // Create minimal realistic data
        let minimal_entries = vec![
            ("/", vec!["GET"]),
            ("/admin/analytics", vec!["GET"]),
            ("/api/test", vec!["GET", "POST"]),
        ];
        
        for (path, methods) in &minimal_entries {
            let mut method_stats = HashMap::new();
            for method in methods {
                method_stats.insert(method.to_string(), (fastrand::u64(1..10), fastrand::f64() * 50.0));
            }
            
            let entry = AnalyticsEntry {
                path: path.to_string(),
                method_stats,
                is_asset: false,
                metadata: vec!["minimal_test".to_string()],
            };
            entry.save().await?;
        }
        
        // Test read performance
        let read_start = Instant::now();
        let analytics = AnalyticsEntry::get_all().await?;
        let read_time = read_start.elapsed();
        
        let total_time = start.elapsed();
        
        Ok(format!(
            "Minimal Data Analysis (no existing data found):\n\
            - Created {} minimal entries\n\
            - Total methods: {}\n\
            - Read time: {:?}\n\
            - Total time: {:?}\n\
            - DIAGNOSIS: {}",
            analytics.len(),
            analytics.iter().map(|e| e.method_stats.len()).sum::<usize>(),
            read_time,
            total_time,
            if read_time > std::time::Duration::from_millis(50) {
                "BASELINE ISSUE - Even minimal data is slow! Check system/database configuration."
            } else {
                "BASELINE OK - Minimal data is fast. Issue likely with data size or concurrent load."
            }
        ))
    }
}

async fn large_db_benchmark() -> Result<String> {
    info!("Creating 200MB+ database to test large database scan performance");
    
    let start = Instant::now();
    
    // Clear existing data
    DB.nuke().await?;
    ROWS_COUNT.store(0, Ordering::SeqCst);
    ANALYTICS_ROWS_COUNT.store(0, Ordering::SeqCst);
    
    // Create large database
    for i in 1..=100000 {
        let entry = Entry::new(i);
        if let Ok(_) = entry.save().await {
            ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    // Create large analytics entries
    for i in 1..=100000 {
        if let Ok(_) = AnalyticsEntry::new(i).save().await {
            ANALYTICS_ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Large Database Simulation:\n\
        - Total entries: {}\n\
        - Total analytics entries: {}\n\
        - Total time: {:?}",
        ROWS_COUNT.load(Ordering::SeqCst),
        ANALYTICS_ROWS_COUNT.load(Ordering::SeqCst),
        total_time
    ))
}

async fn scan_scaling_benchmark() -> Result<String> {
    info!("Testing how scan performance degrades with database size");
    
    let start = Instant::now();
    let mut scan_times = vec![];
    let mut sizes = vec![];
    
    // Start with clean database
    DB.nuke().await?;
    ANALYTICS_ROWS_COUNT.store(0, Ordering::SeqCst);
    
    // Test scan performance at different database sizes
    let test_sizes = vec![100, 500, 1000, 5000, 10000];
    
    for &target_size in &test_sizes {
        // Add more entries to reach target size
        let current_size = ANALYTICS_ROWS_COUNT.load(Ordering::SeqCst);
        for i in (current_size + 1)..=(target_size as u64) {
            let entry = AnalyticsEntry::new(i);
            if let Ok(_) = entry.save().await {
                ANALYTICS_ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }
        
        // Measure scan time at this size
        let scan_start = Instant::now();
        let analytics = AnalyticsEntry::get_all().await?;
        let scan_time = scan_start.elapsed();
        
        scan_times.push(scan_time);
        sizes.push(analytics.len());
        
        info!("Database size: {} entries, scan time: {:?}", analytics.len(), scan_time);
    }
    
    let total_time = start.elapsed();
    
    // Calculate scaling ratio
    let first_time = scan_times[0].as_secs_f64();
    let last_time = scan_times.last().unwrap().as_secs_f64();
    let size_ratio = sizes.last().unwrap() / sizes[0];
    let time_ratio = last_time / first_time;
    
    Ok(format!(
        "Scan Performance vs DB Size:\n\
        - Sizes tested: {:?}\n\
        - Scan times: {:?}\n\
        - Size increase: {}x\n\
        - Time increase: {:.2}x\n\
        - Scaling efficiency: {:.2}% (linear would be 100%)\n\
        - Total time: {:?}",
        sizes, scan_times, size_ratio, time_ratio,
        (size_ratio as f64 / time_ratio) * 100.0,
        total_time
    ))
}

async fn fragmentation_benchmark() -> Result<String> {
    info!("Testing performance with fragmented database (many deletes/updates)");
    
    let start = Instant::now();
    
    // Step 1: Create initial database
    DB.nuke().await?;
    ROWS_COUNT.store(0, Ordering::SeqCst);
    ANALYTICS_ROWS_COUNT.store(0, Ordering::SeqCst);
    
    let initial_size = 10000;
    for i in 1..=initial_size {
        let entry = Entry::new(i);
        if let Ok(_) = entry.save().await {
            ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        
        let analytics_entry = AnalyticsEntry::new(i);
        if let Ok(_) = analytics_entry.save().await {
            ANALYTICS_ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    // Step 2: Measure scan performance before fragmentation
    let before_frag_start = Instant::now();
    let analytics_before = AnalyticsEntry::get_all().await?;
    let before_frag_time = before_frag_start.elapsed();
    
    // Step 3: Create fragmentation through many deletes and inserts
    let fragmentation_start = Instant::now();
    for cycle in 0..50 {
        // Delete every other entry
        for i in (1..=initial_size).step_by(2) {
            if let Ok(Some(entry)) = Entry::get_by_pkey(i).await {
                let _ = entry.remove().await;
                ROWS_COUNT.fetch_sub(1, Ordering::SeqCst);
            }
        }
        
        // Insert new entries (creates fragmentation)
        let offset = initial_size + cycle * 1000;
        for i in 1..=500 {
            let new_id = offset + i;
            let entry = Entry::new(new_id);
            if let Ok(_) = entry.save().await {
                ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }
    }
    let fragmentation_time = fragmentation_start.elapsed();
    
    // Step 4: Measure scan performance after fragmentation
    let after_frag_start = Instant::now();
    let analytics_after = AnalyticsEntry::get_all().await?;
    let after_frag_time = after_frag_start.elapsed();
    
    let total_time = start.elapsed();
    
    Ok(format!(
        "Database Fragmentation Test:\n\
        - Initial entries: {}\n\
        - Analytics entries (unchanged): {}\n\
        - Scan before fragmentation: {:?}\n\
        - Fragmentation operations: {:?}\n\
        - Scan after fragmentation: {:?}\n\
        - Performance degradation: {:.2}x\n\
        - Total time: {:?}",
        initial_size,
        analytics_before.len(),
        before_frag_time,
        fragmentation_time,
        after_frag_time,
        after_frag_time.as_secs_f64() / before_frag_time.as_secs_f64(),
        total_time
    ))
}

async fn production_scenario_benchmark() -> Result<String> {
    info!("Simulating production: 200MB database with small analytics table but lots of other data");
    
    let start = Instant::now();
    
    // Clear existing data
    DB.nuke().await?;
    ROWS_COUNT.store(0, Ordering::SeqCst);
    ANALYTICS_ROWS_COUNT.store(0, Ordering::SeqCst);
    
    // Step 1: Create MASSIVE amounts of non-analytics data (simulating logs, user data, etc.)
    info!("Creating large database with non-analytics data...");
    let large_data_start = Instant::now();
    
    // Create 500K entries with large payloads to simulate a 200MB database
    for i in 1..=500_000 {
        let mut entry = Entry::new(i);
        // Make entries larger to simulate real production data
        entry.optional = Some(format!("Large production data entry {} with lots of text content that would be typical in a real application database with user-generated content, logs, cached data, and other typical database entries that accumulate over time in production systems.", i));
        entry.list = vec![true, false, true, false, true, false]; // Larger list
        
        if let Ok(_) = entry.save().await {
            ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        
        // Log progress every 50K entries
        if i % 50_000 == 0 {
            info!("Created {} entries...", i);
        }
    }
    let large_data_time = large_data_start.elapsed();
    
    // Step 2: Create small, realistic analytics table (like production)
    info!("Creating small analytics table...");
    let analytics_data_start = Instant::now();
    
    // Only 20-50 analytics entries (realistic for most apps)
    for i in 1..=30 {
        let entry = AnalyticsEntry::new(i);
        if let Ok(_) = entry.save().await {
            ANALYTICS_ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    let analytics_data_time = analytics_data_start.elapsed();
    
    // Step 3: Test analytics scan performance in large database
    info!("Testing analytics scan in large database...");
    let mut scan_times = vec![];
    
    for i in 1..=5 {
        let scan_start = Instant::now();
        let analytics = AnalyticsEntry::get_all().await?;
        let scan_time = scan_start.elapsed();
        scan_times.push(scan_time);
        info!("Scan {}: {:?} for {} entries", i, scan_time, analytics.len());
    }
    
    let total_time = start.elapsed();
    let avg_scan_time = scan_times.iter().sum::<std::time::Duration>() / scan_times.len() as u32;
    
    Ok(format!(
        "Production Scenario Test:\n\
        - Large data entries: {}\n\
        - Analytics entries: {}\n\
        - Large data creation: {:?}\n\
        - Analytics data creation: {:?}\n\
        - Avg analytics scan time: {:?}\n\
        - Scan times: {:?}\n\
        - Total time: {:?}\n\
        - DIAGNOSIS: {}",
        ROWS_COUNT.load(Ordering::SeqCst),
        ANALYTICS_ROWS_COUNT.load(Ordering::SeqCst),
        large_data_time,
        analytics_data_time,
        avg_scan_time,
        scan_times,
        total_time,
        if avg_scan_time > std::time::Duration::from_millis(1000) {
            "CONFIRMED - Large database size is causing analytics scan slowdown!"
        } else {
            "Large database size is not causing significant slowdown"
        }
    ))
}
