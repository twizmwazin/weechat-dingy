
use backtrace::Backtrace;
use message;

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

        let buffer = data.get::<u128>(index, "buffer");
        let date = data.get::<u128>(index, "date");
        let date_printed = data.get::<u128>(index, "date_printed");
        let displayed = data.get::<i8>(index, "displayed");
        let highlight = data.get::<i8>(index, "highlight");
        let prefix = data.get::<Option<String>>(index, "prefix");
        let message = data.get::<Option<String>>(index, "message");
        let tags_array = data.get::<Vec<Option<String>>>(index, "tags_array");

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

        Ok(BufferLineAdded {
            buffer: buffer.unwrap(),
            date: date.unwrap(),
            date_printed: date_printed.unwrap(),
            displayed: displayed.unwrap(),
            highlight: highlight.unwrap(),
            tags_array: tags_array.unwrap(),
            prefix: prefix.unwrap(),
            message: message.unwrap()
        })
    }
}

