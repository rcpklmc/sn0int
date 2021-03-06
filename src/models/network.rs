use crate::errors::*;
use crate::fmt::colors::*;
use diesel;
use diesel::prelude::*;
use crate::models::*;
use std::result;


#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="networks"]
pub struct Network {
    pub id: i32,
    pub value: String,
    pub unscoped: bool,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
}

impl Model for Network {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::networks::dsl::*;

        let results = networks.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::networks::dsl::*;

        let query = networks.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::networks::dsl::*;

        diesel::delete(networks.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::networks::dsl::*;

        diesel::delete(networks.filter(id.eq(my_id)))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn value(&self) -> &Self::ID {
        &self.value
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use crate::schema::networks::dsl::*;

        let domain = networks.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::networks::dsl::*;

        let domain = networks.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::networks::dsl::*;

        let domain = networks.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(domain)
    }
}

impl Scopable for Network {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::networks::dsl::*;

        diesel::update(networks.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::networks::dsl::*;

        diesel::update(networks.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl Network {
    fn devices(&self, db: &Database) -> Result<Vec<Device>> {
        let device_ids = NetworkDevice::belonging_to(self)
            .select(network_devices::device_id)
            .load::<i32>(db.db())?;

        device_ids.into_iter()
            .map(|device_id| devices::table
                .filter(devices::id.eq(device_id))
                .first::<Device>(db.db())
            )
            .collect::<result::Result<_, _>>()
            .map_err(Error::from)
    }
}

pub struct PrintableNetwork {
    value: String,
}

impl fmt::Display for PrintableNetwork {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)
    }
}

impl Printable<PrintableNetwork> for Network {
    fn printable(&self, _db: &Database) -> Result<PrintableNetwork> {
        Ok(PrintableNetwork {
            value: self.value.to_string(),
        })
    }
}

pub struct DetailedNetwork {
    id: i32,
    value: String,
    unscoped: bool,
    latitude: Option<f32>,
    longitude: Option<f32>,
    devices: Vec<PrintableDevice>,
}

impl DisplayableDetailed for DetailedNetwork {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.debug::<Green, _>(&self.value)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.latitude)?;
        w.opt_debug::<Yellow, _>(&self.longitude)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        for device in &self.devices {
            w.child(device)?;
        }
        Ok(())
    }
}

display_detailed!(DetailedNetwork);

impl Detailed for Network {
    type T = DetailedNetwork;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let devices = self.devices(db)?.into_iter()
            .map(|sd| sd.printable(db))
            .collect::<Result<_>>()?;

        Ok(DetailedNetwork {
            id: self.id,
            value: self.value.to_string(),
            unscoped: self.unscoped,
            latitude: self.latitude.clone(),
            longitude: self.longitude.clone(),
            devices,
        })
    }
}

#[derive(Insertable)]
#[table_name="networks"]
pub struct NewNetwork<'a> {
    pub value: &'a str,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
}

impl<'a> InsertableStruct<Network> for NewNetwork<'a> {
    fn value(&self) -> &str {
        self.value
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(networks::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl<'a> Upsertable<Network> for NewNetwork<'a> {
    type Update = NetworkUpdate;

    fn upsert(self, existing: &Network) -> Self::Update {
        Self::Update {
            id: existing.id,
            latitude: Self::upsert_opt(self.latitude, &existing.latitude),
            longitude: Self::upsert_opt(self.longitude, &existing.longitude),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="networks"]
pub struct NewNetworkOwned {
    pub value: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
}

impl Printable<PrintableNetwork> for NewNetworkOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableNetwork> {
        Ok(PrintableNetwork {
            value: self.value.to_string(),
        })
    }
}

pub type InsertNetwork = NewNetworkOwned;

impl LuaInsertToNewOwned for InsertNetwork {
    type Target = NewNetworkOwned;

    fn try_into_new(self) -> Result<NewNetworkOwned> {
        Ok(self)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="networks"]
pub struct NetworkUpdate {
    pub id: i32,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
}

impl Upsert for NetworkUpdate {
    fn is_dirty(&self) -> bool {
        self.latitude.is_some() ||
        self.longitude.is_some()
    }

    fn generic(self) -> Update {
        Update::Network(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_network(self)
    }
}

impl Updateable<Network> for NetworkUpdate {
    fn changeset(&mut self, existing: &Network) {
        Self::clear_if_equal(&mut self.latitude, &existing.latitude);
        Self::clear_if_equal(&mut self.longitude, &existing.longitude);
    }

    fn fmt(&self, updates: &mut Vec<String>) {
        Self::push_value(updates, "latitude", &self.latitude);
        Self::push_value(updates, "longitude", &self.longitude);
    }
}
