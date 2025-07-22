mod results;
mod search_settings;

use chrono::{NaiveDate, NaiveTime};
use leptos::{component, html::Input, prelude::*, view, IntoView, ev::SubmitEvent};

use search_settings::SearchSettings;
use results::Results;
use tracing::{debug, instrument};

use crate::structs::{SearchResults, SearchStatus, SearchTarget, SearchTargetBy, SearchTargetMode};

#[server]
#[instrument(skip_all, err(level = "warn"))]
async fn search(target: SearchTarget) -> Result<SearchResults,ServerFnError> {
    use crate::{DefaultData, finder::{MessageFinder, SearchEngine}};
    
    debug!("search: {:?}", target);
    
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    debug!("default: {:?}", default);

    let consumer = supermusr_common::create_default_consumer(
        &default.broker,
        &default.username,
        &default.password,
        &default.consumer_group,
        None)?;

    let mut searcher = SearchEngine::new(
        consumer,
        &default.topics
    );
    let search_result = searcher.search(target)
        .await;
    
    debug!("SearchResult: {search_result:?}");

    Ok(search_result)
}

#[component]
pub(crate) fn Search() -> impl IntoView {
    
    //let search_mode_ref = NodeRef::<Input>::new();
    //let search_by_ref = NodeRef::<Input>::new();
    let date_ref = NodeRef::<Input>::new();
    let time_ref = NodeRef::<Input>::new();
    let number_ref = NodeRef::<Input>::new();
    let channels_ref = NodeRef::<Input>::new();
    let digitiser_ids_ref = NodeRef::<Input>::new();

    let search_fn = move |_: &()| {
        let time = time_ref.get().expect("Time Should exists, this should never fail.").value().parse::<NaiveTime>().expect("");
        let date = date_ref.get().expect("Date Should exists, this should never fail.").value().parse::<NaiveDate>().expect("");
        
        let timestamp = date.and_time(time).and_utc();
        let channels = channels_ref.get().expect("This should never fail.").value().split(",").map(|x|x.parse()).collect::<Result<Vec<_>,_>>().expect("");
        let number = number_ref.get().expect("This should never fail.").value().parse().unwrap_or(1);

        search(SearchTarget {
            mode: SearchTargetMode::Timestamp { timestamp },
            by: SearchTargetBy::ByChannels { channels },
            number
        })
    };
    let search_action = Action::new(search_fn);
    let search_results = search_action.value();

    let on_submit = move |e : SubmitEvent| {
        e.prevent_default();
        search_action.dispatch(());
    };

    view! {
        <form on:submit = on_submit>
            <SearchSettings date_ref time_ref number_ref channels_ref digitiser_ids_ref/>
            {move ||
                search_results.get().map(|search_results|view!{
                    <Results search_results />
                })
            }
        </form>
    }
}