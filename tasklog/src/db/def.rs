use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DeserializeAs, SerializeAs};

use crate::slint_generatedAppWindow::{RecordEntry as UIRecordEntry, RecordState as UIRecordState};
use slint::{Model, ModelRc, VecModel};

pub const RECORD_TABLE: &str = "record";

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RecordEntry {
    pub uuid: String,
    pub start_date: String,
    pub end_date: String,
    pub title: String,
    pub plan: String,
    pub tags: Vec<String>,

    #[serde_as(as = "RecordState")]
    pub state: UIRecordState,
}

impl From<UIRecordEntry> for RecordEntry {
    fn from(entry: UIRecordEntry) -> Self {
        RecordEntry {
            uuid: entry.uuid.into(),
            start_date: entry.start_date.into(),
            end_date: entry.end_date.into(),
            title: entry.title.into(),
            plan: entry.plan.into(),
            state: entry.state,
            tags: entry
                .tags
                .iter()
                .map(|item| item.into())
                .collect::<Vec<_>>(),
        }
    }
}

impl From<RecordEntry> for UIRecordEntry {
    fn from(entry: RecordEntry) -> Self {
        UIRecordEntry {
            uuid: entry.uuid.into(),
            start_date: entry.start_date.into(),
            end_date: entry.end_date.into(),
            title: entry.title.into(),
            plan: entry.plan.into(),
            state: entry.state,

            tags: ModelRc::new(VecModel::from_slice(
                &entry
                    .tags
                    .into_iter()
                    .map(|item| item.into())
                    .collect::<Vec<_>>(),
            )),
        }
    }
}

struct RecordState;
impl SerializeAs<UIRecordState> for RecordState {
    fn serialize_as<S>(source: &UIRecordState, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status = match source {
            UIRecordState::NotStarted => "NotStarted",
            UIRecordState::Running => "Running",
            UIRecordState::Finished => "Finished",
            UIRecordState::Giveup => "Giveup",
            UIRecordState::Timeout => "Timeout",
        };

        serializer.serialize_str(status)
    }
}

impl<'de> DeserializeAs<'de, UIRecordState> for RecordState {
    fn deserialize_as<D>(deserializer: D) -> Result<UIRecordState, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = String::deserialize(deserializer)?;
        let status = match status.as_str() {
            "NotStarted" => UIRecordState::NotStarted,
            "Running" => UIRecordState::Running,
            "Finished" => UIRecordState::Finished,
            "Giveup" => UIRecordState::Giveup,
            "Timeout" => UIRecordState::Timeout,
            _ => unreachable!(),
        };
        Ok(status)
    }
}
