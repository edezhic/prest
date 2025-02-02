use crate::*;
use std::collections::HashMap;

pub(crate) async fn full() -> Result<Markup> {
    let jobs_records = ScheduledJobRecord::get_all().await?;

    #[derive(Default)]
    struct ScheduledJobStat {
        finished_successfully: u32,
        in_progress: u32,
        avg_duration: f64,
        errors: Vec<(NaiveDateTime, String)>,
    }

    let jobs_stats: HashMap<String, ScheduledJobStat> = jobs_records.into_iter().fold(
        HashMap::new(),
        |mut map,
         ScheduledJobRecord {
             name,
             start,
             end,
             error,
             ..
         }| {
            let entry = map.entry(name).or_default();

            if end.is_none() {
                entry.in_progress += 1;
            } else if end.is_some() && error.is_none() {
                let updated_successes = entry.finished_successfully + 1;
                let duration = (end.unwrap() - start).num_milliseconds().abs() as f64;

                let updated_avg_duration =
                    (entry.finished_successfully as f64 * entry.avg_duration + duration)
                        / (updated_successes as f64);

                entry.finished_successfully = updated_successes;
                entry.avg_duration = updated_avg_duration;
            } else if end.is_some() && error.is_some() {
                entry.errors.push((end.unwrap(), error.unwrap()));
            }

            map
        },
    );

    Ok(html! {
        $"w-full" get="/admin/schedule_stats" trigger="load delay:10s" swap-this-no-transition {
            $"font-bold text-lg" {"Scheduled jobs stats"}
            $"w-full text-xs md:text-sm font-mono" {
                @for (name, stats) in jobs_stats {
                    @let duration = format!("{:.1}ms", stats.avg_duration);
                    $"w-full" {b{(name)}": in progress = "(stats.in_progress)", finished = "(stats.finished_successfully)", avg duration = "(duration)}
                    @for (end, error) in stats.errors {
                        p{(end)" - "(error)}
                    }
                }
            }
        }
    })
}
