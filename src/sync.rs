use crate::message;
use backtrace::Backtrace;
use message::WeechatString;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum SyncErrorType {
    InvalidData,
    InvalidId,
    Other,
}

#[derive(Constructor, Debug)]
pub struct SyncError {
    pub error: SyncErrorType,
    pub message: String,
    pub trace: Backtrace,
}

#[derive(Debug)]
pub enum SyncMessage {
    BufferOpened(BufferOpened),
    BufferMoved(BufferMoved),
    BufferMerged(BufferMerged),
    BufferUnmerged(BufferUnmerged),
    BufferHidden(BufferHidden),
    BufferUnhidden(BufferUnhidden),
    BufferRenamed(BufferRenamed),
    BufferTitleChanged(BufferTitleChanged),
    BufferCleared(BufferCleared),
    BufferTypeChanged(BufferTypeChanged),
    BufferLocalvarAdded(BufferLocalvarAdded),
    BufferLocalvarChanged(BufferLocalvarChanged),
    BufferLocalvarRemoved(BufferLocalvarRemoved),
    BufferLineAdded(BufferLineAdded),
    BufferClosing(BufferClosing),
    Nicklist(Nicklist),
    NicklistDiff(NicklistDiff),
    Pong(WeechatString),
    Upgrade,
    UpgradeEnded,
}

pub trait SyncHdataItem<T> {
    fn parse(message: &message::WeechatType) -> Result<Vec<T>, SyncError>;

    // Turn a WeechatType into a vector of messages whose underlying type is T
    // F func is to go from T to SyncMessage (so |a| SyncMessage::Variant(a.clone()) generally)
    // because we can't pass enum variants as parameters.
    fn parse_into<F>(item: &message::WeechatType, func: F) -> Result<Vec<SyncMessage>, SyncError>
    where
        T: SyncHdataItem<T>,
        F: FnMut(&T) -> SyncMessage,
    {
        T::parse(item).map(|vec| vec.iter().map(func).collect::<Vec<_>>())
    }

    fn parse_one(data: &message::Hdata, index: usize) -> Result<T, SyncError>;
}

impl SyncMessage {
    pub fn parse(message: &message::Message) -> Result<Vec<Vec<SyncMessage>>, SyncError> {
        message
            .data
            .iter()
            .map(move |item| SyncMessage::parse_message_item(&message.id, item))
            .collect::<Result<Vec<Vec<SyncMessage>>, SyncError>>()
    }

    pub fn parse_message_item(
        id: &String,
        item: &message::WeechatType,
    ) -> Result<Vec<SyncMessage>, SyncError> {
        match id.as_str() {
            "_buffer_opened" => {
                BufferOpened::parse_into(item, |a| SyncMessage::BufferOpened(a.clone()))
            }
            "_buffer_moved" => {
                BufferMoved::parse_into(item, |a| SyncMessage::BufferMoved(a.clone()))
            }
            "_buffer_merged" => {
                BufferMerged::parse_into(item, |a| SyncMessage::BufferMerged(a.clone()))
            }
            "_buffer_unmerged" => {
                BufferUnmerged::parse_into(item, |a| SyncMessage::BufferUnmerged(a.clone()))
            }
            "_buffer_hidden" => {
                BufferHidden::parse_into(item, |a| SyncMessage::BufferHidden(a.clone()))
            }
            "_buffer_unhidden" => {
                BufferUnhidden::parse_into(item, |a| SyncMessage::BufferUnhidden(a.clone()))
            }
            "_buffer_renamed" => {
                BufferRenamed::parse_into(item, |a| SyncMessage::BufferRenamed(a.clone()))
            }
            "_buffer_title_changed" => {
                BufferTitleChanged::parse_into(item, |a| SyncMessage::BufferTitleChanged(a.clone()))
            }
            "_buffer_cleared" => {
                BufferCleared::parse_into(item, |a| SyncMessage::BufferCleared(a.clone()))
            }
            "_buffer_type_changed" => {
                BufferTypeChanged::parse_into(item, |a| SyncMessage::BufferTypeChanged(a.clone()))
            }
            "_buffer_localvar_added" => BufferLocalvarAdded::parse_into(item, |a| {
                SyncMessage::BufferLocalvarAdded(a.clone())
            }),
            "_buffer_localvar_changed" => BufferLocalvarChanged::parse_into(item, |a| {
                SyncMessage::BufferLocalvarChanged(a.clone())
            }),
            "_buffer_localvar_removed" => BufferLocalvarRemoved::parse_into(item, |a| {
                SyncMessage::BufferLocalvarRemoved(a.clone())
            }),
            "_buffer_line_added" => {
                BufferLineAdded::parse_into(item, |a| SyncMessage::BufferLineAdded(a.clone()))
            }
            "_buffer_closing" => {
                BufferClosing::parse_into(item, |a| SyncMessage::BufferClosing(a.clone()))
            }
            "_nicklist" => Nicklist::parse_into(item, |a| SyncMessage::Nicklist(a.clone())),
            "_nicklist_diff" => {
                NicklistDiff::parse_into(item, |a| SyncMessage::NicklistDiff(a.clone()))
            }
            "_pong" => match item.unwrap::<WeechatString>() {
                Some(ws) => Ok(vec![SyncMessage::Pong(ws)]),
                _ => Err(SyncError {
                    error: SyncErrorType::InvalidId,
                    message: format!("Unexpected type for _pong: {:?}", item).to_owned(),
                    trace: Backtrace::new(),
                }),
            },
            "_upgrade" => Ok(vec![SyncMessage::Upgrade]),
            "_upgrade_ended" => Ok(vec![SyncMessage::UpgradeEnded]),
            _ => Err(SyncError {
                error: SyncErrorType::InvalidId,
                message: format!("Unknown sync id {}", id).to_owned(),
                trace: Backtrace::new(),
            }),
        }
    }
}

// Macro that lets us define weechat sync data structures with their name and fields,
// will generate a parse method that turns an Hdata into a Result<$name>
macro_rules! sync_struct {
    ($name: ident {$($field : ident: $type : ty;)*}) => {

        // Define the structure to have all public fields (for ease of use)
        #[derive(Debug, Clone)]
        pub struct $name {
            $(
                pub $field: $type,
            )*
        }

        impl SyncHdataItem<$name> for $name {
            fn parse(message: &message::WeechatType) -> Result<Vec<$name>, SyncError> {
                match message {
                    message::WeechatType::Hdata(data) => (0..data.len())
                        .map(|i| $name::parse_one(data, i))
                        .collect::<Result<Vec<$name>, SyncError>>(),
                    _ => Err(SyncError {
                        error: SyncErrorType::InvalidData,
                        message: stringify!("Found non-hdata while unwrapping ", $name).to_owned(),
                        trace: Backtrace::new(),
                    }),
                }
            }

            fn parse_one(data: &message::Hdata, index: usize) -> Result<$name, SyncError> {
                // Get all fields out of the hdata with their correct types and make sure they worked
                $(
                    let $field = data.get::<$type>(index, stringify!($field));
                    if $field.is_none() {
                        return Err(SyncError{
                            error: SyncErrorType::InvalidData,
                            message: stringify!($field, " expected Some").to_owned(),
                            trace: Backtrace::new()
                        });
                    }
                )*

                // Then just send off the new object!
                Ok($name{
                    $(
                        $field: $field.unwrap(),
                    )*
                })
            }
        }
    };
}

sync_struct!(BufferOpened {
    number: i32;
    full_name: WeechatString;
    short_name: WeechatString;
    nicklist: i32;
    title: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferMoved {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferMerged {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferUnmerged {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferHidden {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferUnhidden {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferRenamed {
    number: i32;
    full_name: WeechatString;
    short_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferTitleChanged {
    number: i32;
    full_name: WeechatString;
    title: WeechatString;
});

sync_struct!(BufferCleared {
    number: i32;
    full_name: WeechatString;
});

sync_struct!(BufferTypeChanged {
    number: i32;
    full_name: WeechatString;
    r#type: i32;
});

sync_struct!(BufferLocalvarAdded {
    number: i32;
    full_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferLocalvarChanged {
    number: i32;
    full_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferLocalvarRemoved {
    number: i32;
    full_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferLineAdded {
    buffer: u128;
    date: u128;
    date_printed: u128;
    displayed: bool;
    highlight: bool;
    tags_array: Vec<WeechatString>;
    prefix: WeechatString;
    message: WeechatString;
});

sync_struct!(BufferClosing {
    number: i32;
    full_name: WeechatString;
});

sync_struct!(Nicklist {
    group: bool;
    visible: bool;
    level: i32;
    name: WeechatString;
    color: WeechatString;
    prefix: WeechatString;
    prefix_color: WeechatString;
});

sync_struct!(NicklistDiff {
    _diff: i8;
    group: bool;
    visible: bool;
    level: i32;
    name: WeechatString;
    color: WeechatString;
    prefix: WeechatString;
    prefix_color: WeechatString;

});
