mod chemistry;
mod cli;
mod config;
mod cooccurrence;
mod datasets;
mod error;
mod markdown;
mod metadata;
mod population;
mod profiler;
mod records;
mod reports;
mod visuals;

use clap::Parser;
use cli::Cli;
use config::{ProfileConfig, TargetSelection};
use cooccurrence::write_cooccurrence_reports;
use datasets::process_dataset;
use markdown::write_markdown_report;
use profiler::DatasetProfile;
use reports::{ReportPaths, write_reports_index};

use crate::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = ProfileConfig::try_from(cli)?;

    println!("Dataset: {}", config.dataset_name);

    let mut profile = DatasetProfile::default();

    process_dataset(&config.dataset_source, &config.cache_dir, config.record_limit, |record| {
        profile.observe(&record);
        Ok(())
    })
    .await?;

    let target_elements: Vec<String> = match &config.target_selection {
        TargetSelection::One(target) => vec![target.clone()],
        TargetSelection::AllObserved => profile.observed_elements(),
    };

    println!("Target elements: {}", target_elements.join(", "));

    let cooccurrence_report_paths = ReportPaths::prepare(config.reports_root.join("cooccurrence"))?;

    println!(
        "Writing element co-occurrence reports to {}",
        cooccurrence_report_paths.root.display()
    );

    write_cooccurrence_reports(
        &config.dataset_name,
        &profile,
        &cooccurrence_report_paths,
        &config.reports_root,
        &target_elements,
        &config.dataset_source,
    )?;

    for target_element in target_elements {
        let report_dir = config.report_dir_for(&target_element);
        let report_paths = ReportPaths::prepare(&report_dir)?;

        println!("Profiling target element: {target_element}");
        println!("Report directory: {}", report_paths.root.display());

        profile.write_element_reports(&target_element, &report_paths)?;
        write_markdown_report(
            &config.dataset_name,
            &target_element,
            &report_paths,
            &config.dataset_source,
        )?;

        println!("Wrote reports to {}", report_paths.root.display());
    }

    write_reports_index("reports", "REPORTS.md")?;

    println!("Updated REPORTS.md");

    Ok(())
}
