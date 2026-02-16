use anyhow::Result;
use comfy_table::{Cell, Table};
use std::time::Instant;

use crate::api::types::{GenerateRequest, GenerateResponse};
use crate::client::OllamaClient;
use crate::format::{nanos_to_secs, tokens_per_sec};

const DEFAULT_PROMPT: &str =
    "Explain the concept of recursion in programming with a simple example.";

struct ModelResult {
    name: String,
    avg_speed: f64,
    avg_tokens: u64,
    avg_time: f64,
    successful_rounds: u32,
    total_rounds: u32,
}

pub async fn execute(
    client: &OllamaClient,
    models: &[String],
    prompt: Option<&str>,
    rounds: u32,
    save_csv: bool,
    no_unload: bool,
) -> Result<()> {
    let prompt = prompt.unwrap_or(DEFAULT_PROMPT);
    let custom_prompt = prompt != DEFAULT_PROMPT;

    println!("🏁 Ollama Model Benchmark");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Models:  {}", models.join(" "));
    println!("Rounds:  {} per model", rounds);
    if custom_prompt {
        println!("Prompt:  {}", prompt);
    } else {
        println!("Prompt:  [Default] {}", prompt);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let benchmark_start = Instant::now();
    let mut results: Vec<ModelResult> = Vec::new();
    let mut csv_rows: Vec<CsvRow> = Vec::new();

    for model in models {
        println!("🔄 Testing: {}", model);

        let mut total_tokens: u64 = 0;
        let mut total_speed: f64 = 0.0;
        let mut total_time: f64 = 0.0;
        let mut successful_rounds: u32 = 0;

        for round in 1..=rounds {
            if rounds > 1 {
                println!("   Round {}/{}...", round, rounds);
            }

            let request = GenerateRequest {
                model: model.clone(),
                prompt: prompt.to_string(),
                stream: false,
                ..Default::default()
            };

            match client
                .post::<_, GenerateResponse>("/api/generate", &request)
                .await
            {
                Ok(response) => {
                    if let (Some(count), Some(eval_dur)) =
                        (response.eval_count, response.eval_duration)
                    {
                        if count > 0 && eval_dur > 0 {
                            let speed = tokens_per_sec(count, eval_dur);
                            let time_sec = response
                                .total_duration
                                .map(nanos_to_secs)
                                .unwrap_or(0.0);

                            total_tokens += count;
                            total_speed += speed;
                            total_time += time_sec;
                            successful_rounds += 1;

                            if rounds > 1 {
                                println!(
                                    "   ✓ {:.2} tokens/sec, {} tokens, {:.2}s",
                                    speed, count, time_sec
                                );
                            }
                        } else {
                            println!("   ✗ Failed to get valid metrics");
                        }
                    } else {
                        println!("   ✗ Failed to get valid metrics");
                    }
                }
                Err(e) => {
                    println!("   ✗ Request failed: {}", e);
                }
            }
        }

        if successful_rounds > 0 {
            let avg_tokens = total_tokens / successful_rounds as u64;
            let avg_speed = total_speed / successful_rounds as f64;
            let avg_time = total_time / successful_rounds as f64;

            println!("   ✅ Avg: {:.2} tokens/sec", avg_speed);

            results.push(ModelResult {
                name: model.clone(),
                avg_speed,
                avg_tokens,
                avg_time,
                successful_rounds,
                total_rounds: rounds,
            });

            csv_rows.push(CsvRow {
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                model: model.clone(),
                tokens_per_sec: format!("{:.2}", avg_speed),
                avg_tokens: avg_tokens.to_string(),
                avg_time: format!("{:.2}", avg_time),
                rounds_completed: successful_rounds.to_string(),
                total_rounds: rounds.to_string(),
                prompt: prompt.to_string(),
                url: client.url().to_string(),
            });
        } else {
            println!("   ❌ All rounds failed");
            results.push(ModelResult {
                name: model.clone(),
                avg_speed: 0.0,
                avg_tokens: 0,
                avg_time: 0.0,
                successful_rounds: 0,
                total_rounds: rounds,
            });

            csv_rows.push(CsvRow {
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                model: model.clone(),
                tokens_per_sec: "N/A".to_string(),
                avg_tokens: "N/A".to_string(),
                avg_time: "N/A".to_string(),
                rounds_completed: "0".to_string(),
                total_rounds: rounds.to_string(),
                prompt: prompt.to_string(),
                url: client.url().to_string(),
            });
        }

        // Unload model between tests
        if !no_unload {
            println!("   🧹 Unloading model...");
            let unload_req = GenerateRequest {
                model: model.clone(),
                prompt: String::new(),
                stream: false,
                keep_alive: Some(0),
                ..Default::default()
            };
            let _ = client.post_raw("/api/generate", &unload_req).await;
        }

        println!();
    }

    let total_time = benchmark_start.elapsed().as_secs();

    // Results table
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 BENCHMARK RESULTS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_header(vec![
        Cell::new("MODEL"),
        Cell::new("TOKENS/SEC"),
        Cell::new("AVG TOKENS"),
        Cell::new("AVG TIME"),
        Cell::new("STATUS"),
    ]);

    let mut fastest: Option<&ModelResult> = None;

    for result in &results {
        if result.successful_rounds > 0 {
            table.add_row(vec![
                Cell::new(&result.name),
                Cell::new(format!("{:.2}", result.avg_speed)),
                Cell::new(result.avg_tokens.to_string()),
                Cell::new(format!("{:.2}s", result.avg_time)),
                Cell::new(if result.successful_rounds == result.total_rounds {
                    "✓"
                } else {
                    "⚠"
                }),
            ]);

            if fastest.is_none() || result.avg_speed > fastest.unwrap().avg_speed {
                fastest = Some(result);
            }
        } else {
            table.add_row(vec![
                Cell::new(&result.name),
                Cell::new("N/A"),
                Cell::new("N/A"),
                Cell::new("N/A"),
                Cell::new("✗"),
            ]);
        }
    }

    println!("{table}");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if let Some(winner) = fastest {
        println!(
            "🏆 WINNER: {} ({:.2} tokens/sec)",
            winner.name, winner.avg_speed
        );
    }
    println!("⏱️  Total benchmark time: {}s", total_time);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // CSV export
    if save_csv {
        let benchmark_dir = "./benchmarks";
        std::fs::create_dir_all(benchmark_dir)?;
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let csv_path = format!("{}/benchmark_{}.csv", benchmark_dir, timestamp);

        let mut writer = csv::Writer::from_path(&csv_path)?;
        writer.write_record([
            "Timestamp",
            "Model",
            "Tokens/Sec",
            "Avg Tokens",
            "Avg Time (s)",
            "Rounds Completed",
            "Total Rounds",
            "Prompt",
            "URL",
        ])?;

        for row in &csv_rows {
            writer.write_record([
                &row.timestamp,
                &row.model,
                &row.tokens_per_sec,
                &row.avg_tokens,
                &row.avg_time,
                &row.rounds_completed,
                &row.total_rounds,
                &row.prompt,
                &row.url,
            ])?;
        }
        writer.flush()?;

        println!();
        println!("💾 Results saved to: {}", csv_path);
    }

    Ok(())
}

struct CsvRow {
    timestamp: String,
    model: String,
    tokens_per_sec: String,
    avg_tokens: String,
    avg_time: String,
    rounds_completed: String,
    total_rounds: String,
    prompt: String,
    url: String,
}
