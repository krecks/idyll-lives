//! Events
//!
//! The social event log and the milestone events a relationship change implies.

use super::*;

// ---- Events -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventKind {
    Met,
    BecameFriends,
    BecameGoodFriends,
    BecameBestFriends,
    BecameEnemies,
    Crush,
    FirstKiss,
    StartedDating,
    BrokeUp,
    ProposalRejected,
    Fight,
    Jealous,
    Promoted,
    MissedWork,
    /// `a` came over to `b`'s home.
    Visited,
    /// `a` reached level `n` in `skill`.
    SkillUp,
    /// `a`'s household paid `n` in rent and bills.
    PaidRent,
    /// `a`'s household couldn't cover the rent and bills (`n`) and is in debt.
    RentDebt,
    /// `a` upgraded something at home to quality `n`.
    Upgraded,
    /// `a` found a job in `career` at level `n`.
    JobFound,
    /// `a` quit their job in `career` at level `n`.
    QuitJob,
    /// `a` was let go from `career` (level `n`) after missing too many shifts.
    Fired,
    /// `a` reached a goal (`goal`: definition; `n` target, `skill`, `career` category).
    GoalReached,
    /// `a` would like to take on a goal (the player can accept it).
    GoalSuggested,
    /// `a`'s household moved into its home (`a` is the first of them).
    MovedIn,
    /// `a` wore out an object (`n`: its object type) and it broke.
    Broke,
    /// `a` repaired a broken object (`n`: its object type).
    Repaired,
    /// A need ran out on `a` (`n`: the accident, index into content `accidents`).
    Accident,
    /// `a` reached a new life stage (`n`: index into content `life.stages`).
    GrewOlder,
    /// `a` retired from `career` (level `n`).
    Retired,
    /// `a` passed away.
    Died,
    /// `a` moved in with their partner `b`.
    MovedInWith,
    /// `a` moved out of their family's home into a house of their own.
    MovedOut,
    /// `a` left town.
    MovedAway,
    /// `a` and `b` are expecting a baby.
    Expecting,
    /// `a` was born to `b` (and `c`).
    Born,
    /// `b` adopted `a`.
    Adopted,
    /// `a` finished school with a final grade of `n` (0..100).
    Graduated,
    /// `b` didn't let `a` in at the door.
    TurnedAway,
    /// `a` knocked at the door of `b`'s home, but nobody answered.
    NobodyHome,
    /// `a` invited `b` over on the phone, and `b` is coming.
    Invited,
    /// `b` said no on the phone to meeting up with `a` (an invitation, or `a` coming over).
    Declined,
    /// `a` would like to buy an `object` (`n`: its price).
    WantsToBuy,
    /// `a` bought the `object` they wanted.
    Bought,
    /// `a` was told no again about the `object` (`n`: how many times).
    DeniedAgain,
    /// `a`'s household paid `n` for a babysitter for `b` while they're out (at work, visiting...).
    Babysitter,
    /// `a` stayed home from work to look after `b` (no money for a babysitter).
    StayedHome,
    /// `a` is out of the crib (or came to live here) and there's no bed for them at home: fewer
    /// bed places than people past the crib (told to the player's household, which buys its own).
    NoBed,
    /// `a` was told to close the `n` (object kind: put the hot tub's cover on, fold up a bed) and
    /// didn't, as someone else got in or set off for it first (whoever is last out closes it).
    NotClosed,
    /// `a` was told to play a game for two at the `object` (`Interaction::partner`) and had
    /// nobody free to play with, so did its use for one instead (`Interaction::alone`).
    PlayedAlone,
    /// `a` was told to play a game for two at the `object` and had nobody free to play with.
    NoPartner,
    /// `a` set up the new `object` at home (a parcel carried in and put together; `setup.rs`).
    SetUp,
    /// The new `object` `a` wanted was set up for them, nobody at home having got to it.
    SetUpForThem,
    /// There was no room at home for the `object` `a` wanted: it went back, the money back.
    SentBack,
    /// `a` brought their guest `b` something to eat or drink (`n`: the item, an index into
    /// `Content::items`; `hosting.rs`).
    Served,
    /// `a`'s household bought the car `a`'s promotion or new job called for (`n`: its body, an
    /// index into `Content::car_bodies`; `cars/looks.rs`).
    BoughtCar,
    /// `a` fetched the post and had a letter from their friend `b` (`post.rs`).
    GotLetter,
}

impl EventKind {
    /// How much an event matters to someone watching: 0 everyday, 1 notable, 2 a milestone.
    pub fn importance(self) -> u8 {
        use EventKind::*;
        match self {
            FirstKiss | StartedDating | BrokeUp | BecameBestFriends | Promoted | JobFound
            | Fired | GoalReached | MovedIn | GrewOlder | Retired | Died | MovedInWith
            | MovedOut | MovedAway | Expecting | Born | Adopted | Graduated => 2,
            BecameFriends | BecameGoodFriends | BecameEnemies | Crush | ProposalRejected
            | Fight | Jealous | QuitJob | RentDebt | Broke | Accident | TurnedAway | WantsToBuy
            | Bought | DeniedAgain | StayedHome | NoBed | NotClosed | PlayedAlone | NoPartner
            | SetUp | SetUpForThem | SentBack | BoughtCar => 1,
            Met | MissedWork | SkillUp | PaidRent | Upgraded | Visited | GoalSuggested
            | Repaired | NobodyHome | Invited | Declined | Babysitter | Served | GotLetter => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SocialEvent {
    pub id: u64,
    pub tick: u64,
    pub kind: EventKind,
    pub a: u32,
    pub b: u32,
    /// Third party (e.g. who a jealous Sim saw their partner with).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<u32>,
    /// A number for the story (a level, an amount of money).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    /// A one-off payment that came with the event (a promotion bonus). Where an event has both,
    /// `{money}` in its text is this and `{n}` is `n`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pay: Option<i64>,
    /// Skill index, for skill events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<u32>,
    /// Career index, for job events (career category for goal events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub career: Option<u32>,
    /// Goal definition, for goal events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<u32>,
    /// Object type, for things residents want (`Broke` and the like name theirs in `n`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct EventLog {
    next_id: u64,
    events: VecDeque<SocialEvent>,
}

/// The story so far: enough for a few weeks of a whole town.
pub const EVENT_LOG_CAPACITY: usize = 300;

impl EventLog {
    pub fn push(&mut self, tick: u64, kind: EventKind, a: usize, b: usize, c: Option<usize>) {
        self.add(SocialEvent {
            id: 0,
            tick,
            kind,
            a: a as u32,
            b: b as u32,
            c: c.map(|c| c as u32),
            n: None,
            pay: None,
            skill: None,
            career: None,
            goal: None,
            object: None,
        });
    }

    /// The event for a job: one Sim, `n` being the level they hold.
    fn career_event(
        tick: u64,
        kind: EventKind,
        a: usize,
        career: usize,
        n: Option<i64>,
    ) -> SocialEvent {
        SocialEvent {
            id: 0,
            tick,
            kind,
            a: a as u32,
            b: a as u32,
            c: None,
            n,
            pay: None,
            skill: None,
            career: Some(career as u32),
            goal: None,
            object: None,
        }
    }

    /// A job event about one Sim.
    pub fn push_career(
        &mut self,
        tick: u64,
        kind: EventKind,
        a: usize,
        career: usize,
        n: Option<i64>,
    ) {
        self.add(Self::career_event(tick, kind, a, career, n));
    }

    /// A promotion: `a` now works at `level` in `career`, with a one-off `bonus`.
    pub fn push_promotion(&mut self, tick: u64, a: usize, career: usize, level: usize, bonus: i64) {
        let event = Self::career_event(tick, EventKind::Promoted, a, career, Some(level as i64));
        self.add(SocialEvent {
            pay: Some(bonus),
            ..event
        });
    }

    /// A goal event about one Sim.
    pub fn push_goal(&mut self, tick: u64, kind: EventKind, a: usize, goal: &crate::planner::Goal) {
        self.add(SocialEvent {
            id: 0,
            tick,
            kind,
            a: a as u32,
            b: a as u32,
            c: None,
            n: Some(goal.target.round() as i64),
            pay: None,
            skill: goal.skill.map(|s| s as u32),
            career: goal.category.map(|c| c as u32),
            goal: Some(goal.def as u32),
            object: goal.object.map(|o| o as u32),
        });
    }

    /// An event about one Sim, with a number and optionally a skill.
    pub fn push_detail(
        &mut self,
        tick: u64,
        kind: EventKind,
        a: usize,
        n: Option<i64>,
        skill: Option<usize>,
    ) {
        self.add(SocialEvent {
            id: 0,
            tick,
            kind,
            a: a as u32,
            b: a as u32,
            c: None,
            n,
            pay: None,
            skill: skill.map(|s| s as u32),
            career: None,
            goal: None,
            object: None,
        });
    }

    /// An event about two Sims, with a number (an amount of money, an item).
    pub fn push_amount(&mut self, tick: u64, kind: EventKind, a: usize, b: usize, n: i64) {
        self.add(SocialEvent {
            id: 0,
            tick,
            kind,
            a: a as u32,
            b: b as u32,
            c: None,
            n: Some(n),
            pay: None,
            skill: None,
            career: None,
            goal: None,
            object: None,
        });
    }

    /// An event about one Sim and a kind of object (something they want), with a number.
    pub fn push_object(
        &mut self,
        tick: u64,
        kind: EventKind,
        a: usize,
        object: usize,
        n: Option<i64>,
    ) {
        self.add(SocialEvent {
            id: 0,
            tick,
            kind,
            a: a as u32,
            b: a as u32,
            c: None,
            n,
            pay: None,
            skill: None,
            career: None,
            goal: None,
            object: Some(object as u32),
        });
    }

    /// Changes the residents events name (`f` gets each `a`, `b` and `c`).
    pub(crate) fn rewrite_refs(&mut self, mut f: impl FnMut(&mut u32)) {
        for e in &mut self.events {
            f(&mut e.a);
            f(&mut e.b);
            if let Some(c) = &mut e.c {
                f(c);
            }
        }
    }

    fn add(&mut self, mut event: SocialEvent) {
        self.next_id += 1;
        event.id = self.next_id;
        if self.events.len() >= EVENT_LOG_CAPACITY {
            // Everyday events go first, so milestones are remembered longer.
            let old = self.events.len() / 2;
            match self
                .events
                .iter()
                .take(old)
                .position(|e| e.kind.importance() == 0)
            {
                Some(i) => {
                    self.events.remove(i);
                }
                None => {
                    self.events.pop_front();
                }
            }
        }
        self.events.push_back(event);
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &SocialEvent> {
        self.events.iter()
    }

    /// Id of the latest event.
    pub fn last_id(&self) -> u64 {
        self.next_id
    }

    /// The events told after event `id`.
    pub(crate) fn since(&self, id: u64) -> Vec<SocialEvent> {
        self.events.iter().filter(|e| e.id > id).copied().collect()
    }

    /// Takes these events (by id) out of the story (an edit was undone); returns those it had.
    pub(crate) fn take(&mut self, events: &[SocialEvent]) -> Vec<SocialEvent> {
        let mut taken = Vec::new();
        self.events.retain(|e| {
            let gone = events.iter().any(|t| t.id == e.id);
            if gone {
                taken.push(*e);
            }
            !gone
        });
        taken
    }

    /// Tells `events` again (an undone edit was made again), under new ids; returns them.
    pub(crate) fn put_back(&mut self, events: Vec<SocialEvent>) -> Vec<SocialEvent> {
        let from = self.next_id;
        for e in events {
            self.add(e);
        }
        self.since(from)
    }

    /// Rebuilds a log from a save (oldest first); ids carry on after `next_id`.
    pub fn restore(next_id: u64, events: impl IntoIterator<Item = SocialEvent>) -> Self {
        let mut events: VecDeque<SocialEvent> = events.into_iter().collect();
        while events.len() > EVENT_LOG_CAPACITY {
            events.pop_front();
        }
        let next_id = events.iter().map(|e| e.id).fold(next_id, u64::max);
        Self { next_id, events }
    }
}

/// Milestone events implied by a change in the pair's relationship.
pub fn milestone_events(
    before: (&Relationship, &Relationship),
    after: (&Relationship, &Relationship),
) -> Vec<(EventKind, bool)> {
    let mut out = Vec::new();
    let mutual = |x: &Relationship, y: &Relationship| {
        friend_level(x.friendship).min(friend_level(y.friendship))
    };
    let (m0, m1) = (mutual(before.0, before.1), mutual(after.0, after.1));
    if m1 > m0 && m1 >= LEVEL_FRIEND {
        out.push((
            match m1 {
                LEVEL_FRIEND => EventKind::BecameFriends,
                LEVEL_GOOD_FRIEND => EventKind::BecameGoodFriends,
                _ => EventKind::BecameBestFriends,
            },
            false,
        ));
    }
    let worst = |x: &Relationship, y: &Relationship| {
        friend_level(x.friendship)
            .max(friend_level(y.friendship))
            .min(friend_level(x.friendship).min(friend_level(y.friendship)))
    };
    if worst(after.0, after.1) <= LEVEL_ENEMY && worst(before.0, before.1) > LEVEL_ENEMY {
        out.push((EventKind::BecameEnemies, false));
    }
    // Crushes are one-directional: `true` means the target has the crush.
    for (b, a, reversed) in [(before.0, after.0, false), (before.1, after.1, true)] {
        if !a.partners && b.romance < CRUSH_ROMANCE && a.romance >= CRUSH_ROMANCE {
            out.push((EventKind::Crush, reversed));
        }
    }
    out
}
