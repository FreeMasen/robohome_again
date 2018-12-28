use super::Error;
use postgres::{
    Connection,
    TlsMode,
    rows::Row,
};

use chrono::{
    Utc,
    Timelike,
    Datelike,
    Weekday,
    DateTime,
    Duration,
};

use uuid::{
    Uuid,
};

use std::fmt::{
    Debug,
    Display,
    Formatter,
    Result as FmtRes,
};

use ipc::send;

const CONN_STR: &str = include_str!("../../../db_connection");

fn get_connection() -> Result<Connection, Error> {
    let ret = Connection::connect(CONN_STR.trim(), TlsMode::None)?;
    Ok(ret)
}

// **********
// TYPES
// **********
/// A single switch
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Switch {
    pub id: i32,
    pub name: String,
    pub on_code: i32,
    pub off_code: i32,
}
/// A regularly scheduled
/// flip
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScheduledFlip {
    pub id: i32,
    pub hour: i32,
    pub minute: i32,
    pub dow: DayOfTheWeek,
    pub direction: Direction,
    pub kind: FlipKind,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ToSql, FromSql)]
#[postgres(name = "flipkind")]
pub enum FlipKind {
    Custom,
    PreDawn,
    Sunrise,
    Dusk,
    Sunset,
}

/// An instance of a
/// flip action
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Flip {
    pub hour: i32,
    pub minute: i32,
    pub code: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, ToSql, FromSql)]
#[postgres(name = "flipdirection")]
/// A flip direction
pub enum Direction {
    On,
    Off,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct DayOfTheWeek {
    monday: bool,
    tuesday: bool,
    wednesday: bool,
    thursday: bool,
    friday: bool,
    saturday: bool,
    sunday: bool,
}

pub struct Authorization {
    pub created: DateTime<Utc>,
    pub code: Uuid,
}

/// A Authorized user's token
pub enum TokenState {
    /// This token is valid
    Valid,
    /// This token is not valid
    Invalid,
    /// This token is valid but should be
    /// regenerated
    Expired,
}

// **********
// CREATE
// **********
pub fn new_switch(name: &str, on_code: i32, off_code: i32,) -> Result<Switch, Error> {
    let c = get_connection()?;
    let ret = c.query("SELECT id, name, on_code, off_code
                       FROM new_switch($1, $2, $3)",
                      &[&name, &on_code, &off_code])?
        .iter()
        .map(map_switch)
        .next()
        .ok_or(Error::new("nothing returned from new_switch"))?;
    Ok(ret)
}

pub fn new_scheduled_flip(sw_id: i32, hour: i32, minute: i32, dow: DayOfTheWeek, direction: Direction, kind: FlipKind) -> Result<ScheduledFlip, Error> {
    let c = get_connection()?;
    let dow: i32 = dow.into();
    let ret = c.query("SELECT id, hour, minute, dow, direction, kind
                       FROM new_flip($1, $2, $3, $4, $5, $6)",
                      &[&sw_id, &hour, &minute, &dow, &direction, &kind])?
                .iter()
                .map(map_scheduled_flip)
                .next()
                .ok_or(Error::new("nothing returned from new_flip"))?;
    Ok(ret)
}

pub fn new_token() -> Result<Uuid, Error> {
    let c = get_connection()?;
    let ret = c.query("SELECT *
                        FROM new_token()",
                        &[])?
                .iter()
                .map::<Uuid, _>(|r| r.get(0))
                .next()
                .ok_or(Error::new("Unable to create token"))?;
    Ok(ret)
}

// **********
// READ
// **********
pub fn get_flips_this_minute() -> Result<Vec<Flip>, Error> {
    let c = get_connection()?;
    let now = Utc::now();
    let dow: DayOfTheWeek = now.date().weekday().into();
    let dow: i32 = dow.into();
    let ret = c.query("SELECT hour, minute, code
                       FROM get_flips_for_minute($1, $2, $3)",
                        &[&now.time().hour(),
                          &now.time().minute(),
                          &dow])?
                .iter()
                .map(map_flip)
                .collect();

    Ok(ret)
}

pub fn get_flips_for_today() -> Result<Vec<Flip>, Error> {
    let c = get_connection()?;
    let now = super::chrono::Utc::now();
    let dow: DayOfTheWeek = now.date().weekday().into();
    let dow: i32 = dow.into();
    let ret = c.query("SELECT hour, minute, code
                       FROM get_flips_for_day($1)",
                       &[&dow])?
                .iter()
                .map(map_flip)
                .collect();
    Ok(ret)
}

pub fn get_all_switches() -> Result<Vec<Switch>, Error> {
    let c = get_connection()?;
    let ret = c.query("SELECT id, name, on_code, off_code
                       FROM get_all_switches()", &[])?
                    .iter()
                    .map(map_switch).collect();
    Ok(ret)
}

pub fn get_flips_for_switch(switch_id: i32) -> Result<Vec<ScheduledFlip>, Error> {
    println!("get_flips_for_switch {}", switch_id);
    let c = get_connection()?;
    println!("got connection");
    let ret = c.query("SELECT id, hour, minute, dow, direction, kind
                       FROM get_switch_flips($1)",
                       &[&switch_id])?
                .iter()
                .map(map_scheduled_flip)
                .collect();
    Ok(ret)
}

pub fn check_auth(token: &Uuid) -> Result<bool, Error> {
    let c = get_connection()?;
    if let Some(timestamp) = c.query("SELECT *
                        FROM get_auth($1)",
                        &[token])?
                .iter()
                .map::<DateTime<Utc>, _>(|r| r.get(0))
                .next()
    {
        let now = Utc::now();
        let elapsed = timestamp.signed_duration_since(now);
        return Ok(elapsed < Duration::days(1));
    }
    Ok(false)
}

pub fn check_token(token: Uuid, name: &str) -> Result<TokenState, Error> {
    let c = get_connection()?;
    let state = if let Some((db_token, timestamp)) = c.query("SELECT token, timestamp
                        FROM get_token($1)",
                        &[&name])?
                .iter()
                .map::<(Uuid, DateTime<Utc>), _>(|r| (r.get(0), r.get(1)))
                .next()
    {
        if token == db_token {
            if timestamp.signed_duration_since(Utc::now()) < Duration::days(7) {
                TokenState::Valid
            } else {
                TokenState::Expired
            }
        } else {
            TokenState::Invalid
        }
    } else {
        TokenState::Invalid
    };
    Ok(state)
}

// **********
// UPDATE
// **********
pub fn update_switch(id: i32, name: &str, on_code: i32, off_code: i32) -> Result<Switch, Error> {
    let c = get_connection()?;
    let ret = c.query("SELECT id, name, on_code, off_code
                       FROM update_switch($1, $2, $3, $4)",
                       &[&id, &name, &on_code, &off_code])?
                       .iter()
                       .map(map_switch)
                       .next()
                       .ok_or(Error::new("Nothing returned from switch update"))?;
    let _ = send("database", &());
    Ok(ret)
}

pub fn update_flip(id: i32, hour: i32, minute: i32, dow: DayOfTheWeek, direction: Direction, kind: FlipKind) -> Result<ScheduledFlip, Error> {
    let c = get_connection()?;
    let dow: i32 = dow.into();
    let ret = c.query("SELECT id, hour, minute, dow, direction, kind
                       FROM update_flip($1, $2, $3, $4, $5, $6)",
                       &[&id, &hour, &minute, &dow, &direction, &kind])?
                .iter()
                .map(map_scheduled_flip)
                .next()
                .ok_or(Error::new("Nothing returned from flip update"))?;
    let _ = send("database", &());
    Ok(ret)
}


// **********
// DELETE
// **********
pub fn remove_switch(id: i32) -> Result<i32, Error> {
    let c = get_connection()?;
    let ret = c.query("SELECT *
                       FROM remove_switch($1)",
                       &[&id])?
                .iter()
                .next()
                .ok_or(Error::new("Unable to get remove count"))
                .map(|row| row.get(0))?;
    let _ = send("database", &());
    Ok(ret)
}

pub fn remove_flip(id: i32) -> Result<i32, Error> {
    let c = get_connection()?;
    let ret = c.query("SELECT *
                       FROM remove_flip($1)",
                       &[&id])?
                .iter()
                .next()
                .ok_or(Error::new("Unable to get remove count"))
                .map(|row| row.get(0))?;
    let _ = send("database", &());
    Ok(ret)
}
// **********
// MAPPINGS
// **********
fn map_switch(row: Row) -> Switch {
    Switch {
        id: row.get(0),
        name: row.get(1),
        on_code: row.get(2),
        off_code: row.get(3),
    }
}

fn map_scheduled_flip(row: Row) -> ScheduledFlip {
    ScheduledFlip {
        id: row.get(0),
        hour: row.get(1),
        minute: row.get(2),
        dow: row.get::<_, i32>(3).into(),
        direction: row.get(4),
        kind: row.get(5),
    }
}

fn map_flip(row: Row) -> Flip {
    Flip {
        hour: row.get(0),
        minute: row.get(1),
        code: row.get(2),
    }
}

impl Into<i32> for DayOfTheWeek {
    fn into(self) -> i32 {
        let mut ret = 0;
        if self.monday {
            ret += 1;
        }
        if self.tuesday {
            ret += 2;
        }
        if self.wednesday {
            ret += 4;
        }
        if self.thursday {
            ret += 8;
        }
        if self.friday {
            ret += 16;
        }
        if self.saturday {
            ret += 32;
        }
        if self.sunday {
            ret += 64;
        }
        ret
    }
}

impl From<i32> for DayOfTheWeek {
    fn from(v: i32) -> DayOfTheWeek {
        DayOfTheWeek {
            monday:    v &  1 > 0,
            tuesday:   v &  2 > 0,
            wednesday: v &  4 > 0,
            thursday:  v &  8 > 0,
            friday:    v & 16 > 0,
            saturday:  v & 32 > 0,
            sunday:    v & 64 > 0,
        }
    }
}

impl From<Weekday> for DayOfTheWeek {
    fn from(wd: Weekday) -> DayOfTheWeek {
        match wd {
            Weekday::Mon => 1i32,
            Weekday::Tue => 2i32,
            Weekday::Wed => 4i32,
            Weekday::Thu => 8i32,
            Weekday::Fri => 16i32,
            Weekday::Sat => 32i32,
            Weekday::Sun => 64i32,
        }.into()
    }
}

impl From<bool> for Direction {
    fn from(i: bool) -> Direction {
        match i {
            false => Direction::Off,
            true => Direction::On,
        }
    }
}

impl Into<bool> for Direction {
    fn into(self) -> bool {
        match self {
            Direction::Off => false,
            Direction::On => true,
        }
    }
}

impl Display for DayOfTheWeek {
    fn fmt(&self, f: &mut Formatter) -> FmtRes {
        let mut s = String::with_capacity(7);
        if self.monday {
            s.push('M');
        }
        if self.tuesday {
            s.push('T');
        }
        if self.wednesday {
            s.push('W');
        }
        if self.thursday {
            s.push('R');
        }
        if self.friday {
            s.push('F')
        }
        if self.saturday {
            s.push('S');
        }
        if self.saturday {
            s.push('U');
        }
        write!(f, "{}", s)
    }
}

impl Debug for DayOfTheWeek {
    fn fmt(&self, f: &mut Formatter) -> FmtRes {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn dow_int() {
        for i in 0..128 {
            let dow: DayOfTheWeek = i.into();
            let back: i32 = dow.into();
            assert_eq!(i, back);
        }
    }

    #[test]
    fn cron_dow() {
        let weekdays = vec![
            (Weekday::Mon, DayOfTheWeek {
                monday: true,
                tuesday: false,
                wednesday: false,
                thursday: false,
                friday: false,
                saturday: false,
                sunday: false,
            }),
            (Weekday::Tue, DayOfTheWeek {
                monday: false,
                tuesday: true,
                wednesday: false,
                thursday: false,
                friday: false,
                saturday: false,
                sunday: false,
            }),
            (Weekday::Wed, DayOfTheWeek{
                monday: false,
                tuesday: false,
                wednesday: true,
                thursday: false,
                friday: false,
                saturday: false,
                sunday: false,
            }),
            (Weekday::Thu, DayOfTheWeek {
                monday: false,
                tuesday: false,
                wednesday: false,
                thursday: true,
                friday: false,
                saturday: false,
                sunday: false,
            }),
            (Weekday::Fri, DayOfTheWeek {
                monday: false,
                tuesday: false,
                wednesday: false,
                thursday: false,
                friday: true,
                saturday: false,
                sunday: false,
            }),
            (Weekday::Sat, DayOfTheWeek {
                monday: false,
                tuesday: false,
                wednesday: false,
                thursday: false,
                friday: false,
                saturday: true,
                sunday: false,
            }),
            (Weekday::Sun, DayOfTheWeek {
                monday: false,
                tuesday: false,
                wednesday: false,
                thursday: false,
                friday: false,
                saturday: false,
                sunday: true,
            }),
        ];
        for (wd, dow) in weekdays {
            let d: DayOfTheWeek = wd.into();
            assert_eq!(d, dow);
        }
    }

    #[test]
    fn db_round_trip() {
        println!("Creating test switch");
        let sw1 = new_switch("test switch", 44444, 55555).expect("failed to insert new switch");
        println!("Updating test switch");
        let sw2 = update_switch(sw1.id, "updated test switch", sw1.on_code, 99999).expect("failed to update switch");
        println!("Checking switches don't match");
        assert!(sw1 != sw2);
        assert!(sw1.id == sw2.id);
        println!("Creating new flip");
        let fl1 = new_scheduled_flip(sw2.id, 10, 0, 64.into(), Direction::On, FlipKind::Custom).expect("failed to insert new flip");
        println!("Updating flip");
        let fl2 = update_flip(fl1.id, fl1.hour, 30, 128.into(), Direction::Off, FlipKind::PreDawn).expect("failed to update flip");
        println!("Checking flips don't match");
        assert!(fl1 != fl2);
        assert!(fl1.id == fl2.id);
        println!("Removing flip");
        remove_flip(fl1.id).expect("failed to remove flip");
        println!("Removing switch");
        remove_switch(sw1.id).expect("failed to remove switch");
    }
}
