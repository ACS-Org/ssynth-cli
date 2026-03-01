use anyhow::{Context, Result};

use crate::cli::JobStatusArgs;
use crate::client::{check_response, ApiClient};
use crate::models::JobDetailResponse;
use crate::output::{
    format_duration_secs, format_time, new_table, print_output, short_uuid, OutputMode,
};

pub async fn run(args: &JobStatusArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    loop {
        let job = fetch_job_detail(&args.job_id, client).await?;
        let is_terminal = is_terminal_status(&job.status);

        print_output(mode, &job, |j| {
            let mut table = new_table();
            table.set_header(["FIELD", "VALUE"]);
            table.add_row(["Job ID", &j.id.to_string()]);
            table.add_row(["Status", &j.status]);
            table.add_row(["Project", &short_uuid(&j.project_id)]);
            table.add_row(["Target", &short_uuid(&j.target_id)]);
            table.add_row(["Top Module", &j.top_module]);
            table.add_row(["Seeds", &j.search_seeds.to_string()]);
            table.add_row(["Pick", &j.search_pick]);
            table.add_row(["Priority", &j.compute_priority]);
            table.add_row(["Created", &format_time(&j.created_at)]);
            table.add_row(["Updated", &format_time(&j.updated_at)]);
            table
        });

        if mode == OutputMode::Human && !job.runs.is_empty() {
            eprintln!();
            let mut run_table = new_table();
            run_table.set_header(["RUN", "SEED", "STATUS", "TIMING", "LUTs", "FFs", "WINNER"]);
            for r in &job.runs {
                run_table.add_row([
                    short_uuid(&r.id),
                    r.seed.to_string(),
                    r.status.clone(),
                    r.timing_mhz
                        .map_or("-".to_string(), |v| format!("{v:.1} MHz")),
                    r.area_luts.map_or("-".to_string(), |v| v.to_string()),
                    r.area_ffs.map_or("-".to_string(), |v| v.to_string()),
                    if r.is_winner {
                        "yes".to_string()
                    } else {
                        "-".to_string()
                    },
                ]);

                // Show steps for each run
                for s in &r.steps {
                    run_table.add_row([
                        format!("  {}", s.step_name),
                        String::new(),
                        s.status.clone(),
                        String::new(),
                        String::new(),
                        format_duration_secs(s.duration_secs),
                        String::new(),
                    ]);
                }
            }
            println!("{run_table}");
        }

        if !args.watch || is_terminal {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        // Clear screen for updated view in watch mode
        eprint!("\x1B[2J\x1B[H");
    }

    Ok(())
}

pub async fn fetch_job_detail(job_id: &str, client: &ApiClient) -> Result<JobDetailResponse> {
    let resp = client
        .get(&format!("/v1/jobs/{job_id}"))
        .send()
        .await
        .context("Failed to fetch job")?;
    let resp = check_response(resp).await?;
    let job: JobDetailResponse = resp.json().await.context("Failed to parse job")?;
    Ok(job)
}

fn is_terminal_status(status: &str) -> bool {
    matches!(status, "completed" | "failed" | "cancelled")
}
