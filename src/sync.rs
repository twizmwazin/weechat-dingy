
use backtrace::Backtrace;
use message;
use std::clone::Clone;

#[derive(Debug)]
pub enum SyncErrorType {
    InvalidData,
    Other,
}

#[derive(Constructor, Debug)]
pub struct SyncError {
    pub error: SyncErrorType,
    pub message: String,
    pub trace: Backtrace,
}

#[derive(Debug)]
pub struct BufferLineAdded {
    pub buffer: u128,
    pub date: u128,
    pub date_printed: u128,
    pub displayed: i8,
    pub highlight: i8,
    pub tags_array: Vec<Option<String>>,
    pub prefix: Option<String>,
    pub message: Option<String>
}

impl BufferLineAdded {
    pub fn parse(data: &message::Hdata, index: usize) -> Result<BufferLineAdded, SyncError> {

        let buffer = data.get(index, "buffer".to_owned()).and_then(|a| a.unwrap_u128());
        let date = data.get(index, "date".to_owned()).and_then(|a| a.unwrap_u128());
        let date_printed = data.get(index, "date_printed".to_owned()).and_then(|a| a.unwrap_u128());
        let displayed = data.get(index, "displayed".to_owned()).and_then(|a| a.unwrap_i8());
        let highlight = data.get(index, "highlight".to_owned()).and_then(|a| a.unwrap_i8());
        let prefix = data.get(index, "prefix".to_owned()).and_then(|a| a.unwrap_string());
        let message = data.get(index, "message".to_owned()).and_then(|a| a.unwrap_string());
        //Convert from WeechatType to Option<Vec<Option<&Option<String>>>>
        // Yes that type is a nightmare... it's because weechat strings are stored as Option<String>
        // And they could not exist, so Option<Option<String>>, and the Vec could not exist.
        // Probably some way to make this cleaner in the future
        let tags_array = data.get(index, "tags_array".to_owned())
            .and_then(|a| a.unwrap_array())
            .map(|array| array.into_iter().map(|item| item.unwrap_string()).collect::<Vec<_>>());

        // TODO: Make this cleaner
macro_rules! assert_some {
    ($var: ident, $msg: expr) => {
        if $var.is_none() {
            return Err(SyncError{
                error: SyncErrorType::InvalidData,
                message: $msg.to_owned(),
                trace: Backtrace::new()
            });
        }
    };
}
        //Make sure everything exists
        assert_some!(buffer, "`buffer` parameter undefined");
        assert_some!(date, "`date` parameter undefined");
        assert_some!(date_printed, "`date_printed` parameter undefined");
        assert_some!(displayed, "`displayed` parameter undefined");
        assert_some!(highlight, "`highlight` parameter undefined");
        assert_some!(tags_array, "`tags_array` parameter undefined");
        assert_some!(prefix, "`prefix` parameter undefined");
        assert_some!(message, "`message` parameter undefined");

        //Check all strings in tags_array
        for item in tags_array.as_ref().unwrap() {
            assert_some!(item, "`tags_array[]` undefined");
        }

        //Unwrap the Option<Vec<Option<&Option<String>>>> into a Vec<Option<String>>
        let real_tags_array = tags_array.unwrap().into_iter().map(|item| item.unwrap().clone()).collect::<Vec<_>>();
        
        Ok(BufferLineAdded {
            buffer: *buffer.unwrap(),
            date: *date.unwrap(),
            date_printed: *date_printed.unwrap(),
            displayed: *displayed.unwrap(),
            highlight: *highlight.unwrap(),
            tags_array: real_tags_array,
            prefix: prefix.unwrap().clone(),
            message: message.unwrap().clone()
        })
    }
}

