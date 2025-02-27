use crate::create::create;
use crate::parse_info::ParseInfo;
use crate::util::finalize;

pub fn build(parsed_info: &ParseInfo) {
    create(parsed_info);
    finalize(parsed_info);
}
