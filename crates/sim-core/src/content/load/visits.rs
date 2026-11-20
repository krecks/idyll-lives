//! Visits: how long guests stay, what they do, and the door.

use super::*;

pub(super) fn parse(
    v: &VisitRaw,
    socials: &[SocialDef],
    visit_tag: TagMask,
    need_array: &impl Fn(&HashMap<String, f32>, &str) -> Result<[f32; MAX_NEEDS], Error>,
    anim_index: &impl Fn(&str) -> Option<usize>,
    feeling: &impl Fn(&Option<String>, &str) -> Result<Option<usize>, Error>,
) -> Result<VisitRules, Error> {
    let visits = VisitRules {
        gains: need_array(&v.needs, "visits")?,
        hours: v.hours.unwrap_or(3.0),
        directed_hours: v.directed_hours.unwrap_or(v.hours.unwrap_or(3.0) * 2.0),
        min_friendship: v.min_friendship.unwrap_or(15.0),
        earliest_hour: v.earliest_hour.unwrap_or(9.0),
        latest_hour: v.latest_hour.unwrap_or(21.0),
        outing_minutes: v.outing_minutes.unwrap_or(90.0).max(0.0),
        outing_home_by: v.outing_home_by.unwrap_or(23.0).clamp(0.0, 24.0),
        least_minutes: v.least_minutes.unwrap_or(0.0).max(0.0),
        again_minutes: v.again_minutes.unwrap_or(0.0).max(0.0),
        tags: visit_tag,
        company: v.company.unwrap_or(0.0).max(0.0),
        bored_minutes: v.bored_minutes.map(|m| m.max(1.0)),
        farewell: v
            .farewell
            .iter()
            .map(|id| {
                socials
                    .iter()
                    .position(|s| &s.id == id)
                    .ok_or_else(|| Error::new(format!("visits.farewell: unknown social '{id}'")))
            })
            .collect::<Result<_, _>>()?,
        farewell_minutes: v.farewell_minutes.unwrap_or(10.0).max(1.0),
        offer: match &v.offer {
            None => None,
            Some(o) => {
                let social = socials
                    .iter()
                    .position(|s| s.id == o.social)
                    .ok_or_else(|| {
                        Error::new(format!("visits.offer: unknown social '{}'", o.social))
                    })?;
                if socials[social].success.effect != crate::social::Effect::Serve {
                    return Err(Error::new(format!(
                        "visits.offer: social '{}' must have \"effect\": \"serve\" on success",
                        o.social
                    )));
                }
                Some(OfferRules {
                    social,
                    after_minutes: o.after_minutes.unwrap_or(15.0).max(0.0),
                    again_minutes: o.again_minutes.unwrap_or(120.0).max(1.0),
                    hand_anim: match &o.hand_anim {
                        None => None,
                        Some(a) => Some(anim_index(a).ok_or_else(|| {
                            Error::new(format!(
                                "visits.offer: unknown anim '{a}' (declare it in \"animations\")"
                            ))
                        })?),
                    },
                })
            }
        },
        door: match &v.door {
            None => None,
            Some(d) => Some(DoorRules {
                wait_minutes: d.wait_minutes.unwrap_or(10.0).max(1.0),
                welcome_friendship: d.welcome_friendship.unwrap_or(-20.0),
                greet: match &d.greet {
                    None => None,
                    Some(id) => {
                        Some(socials.iter().position(|s| &s.id == id).ok_or_else(|| {
                            Error::new(format!("visits.door: unknown social '{id}'"))
                        })?)
                    }
                },
                turned_away: feeling(&d.turned_away_feeling, "visits.door")?,
                knock_anim: match &d.knock_anim {
                    None => None,
                    Some(a) => Some(anim_index(a).ok_or_else(|| {
                        Error::new(format!(
                            "visits.door: unknown anim '{a}' (declare it in \"animations\")"
                        ))
                    })?),
                },
            }),
        },
    };
    Ok(visits)
}
