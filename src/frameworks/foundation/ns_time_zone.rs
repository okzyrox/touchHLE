/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSTimeZone`.

use crate::frameworks::foundation::{ns_string, NSInteger, NSTimeInterval};
use crate::objc::{autorelease, id, nil, release, retain, ClassExports, HostObject, NSZonePtr};
use crate::{msg, objc_classes};

#[derive(Default)]
pub struct State {
    system_time_zone: Option<id>,
}

pub type TimeZoneOffsets = &'static [(&'static str, NSTimeInterval)];
pub type TimeZoneNames = &'static [(&'static str, &'static str)];

pub struct NSTimeZoneHostObject {
    // NSString*
    pub time_zone: id,
    pub gmt_offset: NSInteger,
    pub daylight_savings: bool,
}
impl HostObject for NSTimeZoneHostObject {}

// TODO: maybe a better way to represent this data?
pub const TIME_ZONE_OFFSETS: TimeZoneOffsets = &[
    ("GMT", 0.0),
    ("UTC", 0.0),
    ("America/Los_Angeles", -8.0 * 3600.0),
];

pub const TIME_ZONE_NAMES: TimeZoneNames = &[
    ("GMT", "GMT"),
    ("UTC", "UTC"),
    ("America/Los_Angeles", "PST"),
];

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSTimeZone: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSTimeZoneHostObject {
        time_zone: nil,
        gmt_offset: 0,
        daylight_savings: false
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)timeZoneWithName:(id)tz_name {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithName:tz_name];
    autorelease(env, new)
}

+ (id)timeZoneWithAbbreviation:(id)abbreviation {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithAbbreviation:abbreviation];
    autorelease(env, new)
}

+ (id)timeZoneForSecondsFromGMT:(NSInteger)seconds {
    let new: id = msg![env; this alloc];
    let gmt_offset: NSInteger = seconds;
    // According to the docs, "time zones created (will) never have daylight savings",
    // even if it matches with an existing timezones offset.
    let daylight_savings: bool = false;
    let new: id = msg![env; new initWithName:nil gmtOffset:gmt_offset daylightSavings:daylight_savings];
    autorelease(env, new)
}

+ (id)localTimeZone {
    // According to docs, `localTimeZone` is not cached in contrast to
    // `systemTimeZone`
    let gmt_tz_name: id = ns_string::get_static_str(env, "GMT");
    msg![env; this timeZoneWithName:gmt_tz_name]
}

+ (id)systemTimeZone {
    if let Some(system_time_zone) = env.framework_state.foundation.ns_time_zone.system_time_zone {
        system_time_zone
    } else {
        let new: id = msg![env; this alloc];
        let gmt_tz_name: id = ns_string::get_static_str(env, "GMT");
        let new: id = msg![env; new initWithAbbreviation:gmt_tz_name];
        env.framework_state.foundation.ns_time_zone.system_time_zone = Some(new);
        new
    }
}

+ (id)defaultTimeZone {
    // TODO: implement setting a default time zone
    msg![env; this systemTimeZone]
}

- (())dealloc {
    let tz_name = env.objc.borrow_mut::<NSTimeZoneHostObject>(this).time_zone;
    release(env, tz_name);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)initWithName:(id)tz_name { // NSString *
    assert_ne!(tz_name, nil);
    retain(env, tz_name);
    let tz_name = TIME_ZONE_NAMES.iter().find(|(name, _)| {
        let name_str = ns_string::to_rust_string(env, tz_name);
        name_str == *name
    }).unwrap_or(&("GMT", "GMT"));

    let tz_offset = TIME_ZONE_OFFSETS.iter().find(|(name, _)| {
        tz_name.0 == *name
    }).unwrap_or(&("GMT", 0.0));

    let gmt_offset: NSInteger = tz_offset.1 as NSInteger;
    let daylight_savings: bool = false;
    env.objc.borrow_mut::<NSTimeZoneHostObject>(this).time_zone = ns_string::get_static_str(env, tz_name.1);
    env.objc.borrow_mut::<NSTimeZoneHostObject>(this).gmt_offset = gmt_offset;
    env.objc.borrow_mut::<NSTimeZoneHostObject>(this).daylight_savings = daylight_savings;
    this
}

- (id)initWithAbbreviation:(id)abbreviation { // NSString *
    let tz_name = TIME_ZONE_NAMES.iter().find(|(_, abbr)| {
        let abbr_str = ns_string::to_rust_string(env, abbreviation);
        abbr_str == *abbr
    }).unwrap_or(&("GMT", "GMT"));
    let tz_name: id = ns_string::get_static_str(env, tz_name.0);
    msg![env; this initWithName:tz_name]
}

- (id)name {
    env.objc.borrow_mut::<NSTimeZoneHostObject>(this).time_zone
}

- (id)abbreviation {
    let tz_name = env.objc.borrow_mut::<NSTimeZoneHostObject>(this).time_zone;
    let tz_name_string = ns_string::to_rust_string(env, tz_name);
    let tz_abbreviation = TIME_ZONE_NAMES.iter().find(|(name, _)| {
        tz_name_string == *name
    }).unwrap_or(&("GMT", "GMT"));
    let tz_abbreviation = ns_string::get_static_str(env, tz_abbreviation.1);
    tz_abbreviation
}

- (NSInteger)secondsFromGMT {
    env.objc.borrow_mut::<NSTimeZoneHostObject>(this).gmt_offset
}

- (NSInteger)secondsFromGMTForDate:(id)_date {
    // TODO: implement in accordance with daylight savings stuff
    msg!(env; this secondsFromGMT)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

@end

};
