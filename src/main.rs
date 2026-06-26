mod chemistry;
mod config;
mod cooccurence;
mod datasets;
mod error;
mod markdown;
mod metadata;
mod population;
mod profiler;
mod records;
mod reports;
mod visuals;

use std::collections::{BTreeMap, BTreeSet};

use config::{ProfileConfig, TargetSelection};
use cooccurence::{CooccurrenceProfile, write_cooccurrence_reports};
use datasets::process_dataset;
use markdown::write_markdown_report;
use profiler::ElementProfilerState;
use reports::{ReportPaths, write_reports_index};

use crate::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = ProfileConfig::from_args()?;

    println!("Dataset: {}", config.dataset_name);

    // 1. Setup accumulators
    let mut cooccurrence = CooccurrenceProfile::default();
    let mut element_profilers: BTreeMap<String, ElementProfilerState> = BTreeMap::new();
    let mut observed_all = BTreeSet::new();

    // 2. Stream the dataset
    process_dataset(&config.dataset_name, &config.dataset_source, &config.cache_dir, |record| {
        cooccurrence.observe(&record);

        match &config.target_selection {
            TargetSelection::One(target) => {
                let profiler = element_profilers.entry(target.clone()).or_default();
                profiler.observe(&record, target);
            }
            TargetSelection::AllObserved => {
                for element in record.element_counts.keys() {
                    observed_all.insert(element.clone());
                    let profiler = element_profilers.entry(element.clone()).or_default();
                    profiler.observe(&record, element);
                }
            }
        }
        Ok(())
    })
    .await?;

    let target_elements: Vec<String> = match &config.target_selection {
        TargetSelection::One(target) => vec![target.clone()],
        TargetSelection::AllObserved => observed_all.into_iter().collect(),
    };

    println!("Target elements: {}", target_elements.join(", "));

    let cooccurrence_report_paths = ReportPaths::prepare(config.reports_root.join("cooccurrence"))?;

    println!(
        "Writing element co-occurrence reports to {}",
        cooccurrence_report_paths.root.display()
    );

    // 3. Post-stream processing
    write_cooccurrence_reports(
        &config.dataset_name,
        &cooccurrence,
        &cooccurrence_report_paths,
        &config.reports_root,
        &target_elements,
    )?;

    for target_element in target_elements {
        if let Some(profiler) = element_profilers.get(&target_element) {
            let report_dir = config.report_dir_for(&target_element);
            let report_paths = ReportPaths::prepare(&report_dir)?;

            println!("Profiling target element: {target_element}");
            println!("Report directory: {}", report_paths.root.display());

            profiler.write_reports(&target_element, &report_paths)?;
            write_markdown_report(&config.dataset_name, &target_element, &report_paths)?;

            println!("Wrote reports to {}", report_paths.root.display());
        }
    }

    write_reports_index("reports", "REPORTS.md")?;

    println!("Updated REPORTS.md");

    Ok(())
}
