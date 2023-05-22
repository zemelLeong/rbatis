use once_cell::sync::Lazy;
use serde::ser::SerializeStruct;
use serde::{Deserializer, Serializer};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

///Snowflakes algorithm
#[derive(Debug)]
pub struct Snowflake {
    pub epoch: i64,
    pub worker_id: i64,
    pub datacenter_id: i64,
    pub sequence: AtomicI64,
}

impl serde::Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut s = serializer.serialize_struct("Snowflake", 5)?;
        s.serialize_field("epoch", &self.epoch)?;
        s.serialize_field("worker_id", &self.worker_id)?;
        s.serialize_field("datacenter_id", &self.datacenter_id)?;
        s.serialize_field("sequence", &self.sequence.load(Ordering::Relaxed))?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Snowflake {
            pub epoch: i64,
            pub worker_id: i64,
            pub datacenter_id: i64,
            pub sequence: i64,
        }
        let proxy = Snowflake::deserialize(deserializer)?;
        Ok(self::Snowflake {
            epoch: proxy.epoch,
            worker_id: proxy.worker_id,
            datacenter_id: proxy.datacenter_id,
            sequence: AtomicI64::from(proxy.sequence),
        })
    }
}

impl Clone for Snowflake {
    fn clone(&self) -> Self {
        let sequence = self.sequence.load(Ordering::Relaxed);
        Self {
            epoch: self.epoch,
            worker_id: self.worker_id,
            datacenter_id: self.datacenter_id,
            sequence: AtomicI64::new(sequence),
        }
    }
}

impl Snowflake {
    pub fn default() -> Snowflake {
        Snowflake {
            epoch: 1_564_790_400_000,
            worker_id: 1,
            datacenter_id: 1,
            sequence: AtomicI64::new(0),
        }
    }

    pub fn new(epoch: i64, worker_id: i64, datacenter_id: i64) -> Snowflake {
        Snowflake {
            epoch,
            worker_id,
            datacenter_id,
            sequence: AtomicI64::new(0),
        }
    }

    pub fn set_epoch(&mut self, epoch: i64) -> &mut Self {
        self.epoch = epoch;
        self
    }

    pub fn set_worker_id(&mut self, worker_id: i64) -> &mut Self {
        self.worker_id = worker_id;
        self
    }

    pub fn set_datacenter_id(&mut self, datacenter_id: i64) -> &mut Self {
        self.datacenter_id = datacenter_id;
        self
    }

    pub fn generate(&self) -> i64 {
        let timestamp = self.get_time();
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        (timestamp << 22)
            | (self.worker_id << 17)
            | (self.datacenter_id << 12)
            | sequence
    }

    fn get_time(&self) -> i64 {
        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
        since_the_epoch.as_millis() as i64 - self.epoch
    }
}

pub static SNOWFLAKE: Lazy<Snowflake> = Lazy::new(|| Snowflake::default());

///gen new snowflake_id
pub fn new_snowflake_id() -> i64 {
    SNOWFLAKE.generate()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::snowflake::{new_snowflake_id, Snowflake};

    #[test]
    fn test_gen() {
        println!("{}", new_snowflake_id());
    }

    #[test]
    fn test_ser_de() {
        let s = Snowflake::default();
        s.generate();
        let data = serde_json::to_string(&s).unwrap();
        println!("source:{}", serde_json::to_string(&s).unwrap());
        let r: Snowflake = serde_json::from_str(&data).unwrap();
        println!("new:{}", serde_json::to_string(&r).unwrap());
    }

    #[test]
    fn test_race() {
        let size = 1000;
        let mut v1: Vec<i64> = Vec::with_capacity(size);
        let mut v2: Vec<i64> = Vec::with_capacity(size);
        let mut v3: Vec<i64> = Vec::with_capacity(size);
        let mut v4: Vec<i64> = Vec::with_capacity(size);
        println!(
            "v1 len:{},v2 len:{},v3 len:{},v4 len:{}",
            v1.len(),
            v2.len(),
            v3.len(),
            v4.len()
        );
        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..size {
                    v1.push(new_snowflake_id());
                }
            });
            s.spawn(|| {
                for _ in 0..size {
                    v2.push(new_snowflake_id());
                }
            });
            s.spawn(|| {
                for _ in 0..size {
                    v3.push(new_snowflake_id());
                }
            });
            s.spawn(|| {
                for _ in 0..size {
                    v4.push(new_snowflake_id());
                }
            });
        });

        println!(
            "v1 len:{},v2 len:{},v3 len:{},v4 len:{}",
            v1.len(),
            v2.len(),
            v3.len(),
            v4.len()
        );
        let mut all: Vec<i64> = Vec::with_capacity(size * 4);
        all.append(&mut v1);
        all.append(&mut v2);
        all.append(&mut v3);
        all.append(&mut v4);


        let mut id_map: HashMap<i64, i64> = HashMap::with_capacity(all.len());
        for id in all {
            id_map
                .entry(id)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        for (_, v) in id_map {
            assert_eq!(v <= 1, true);
        }
    }

    #[test]
    fn test_generate_no_clock_back() {
        let snowflake = Snowflake::default();
        let id1 = snowflake.generate();
        let id2 = snowflake.generate();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_clock_back() {
        let mut snowflake = Snowflake::default();
        let initial_timestamp = snowflake.get_time();
        snowflake.set_epoch(initial_timestamp - 100000000000000000); // 设置一个较早的 epoch

        // 生成一个初始 ID
        let initial_id = snowflake.generate();

        // 模拟时钟回拨，将时间设置为初始 ID 的时间戳减去一段时间
        snowflake.set_epoch(initial_timestamp - 200000000000000000);

        // 生成新的 ID
        let new_id = snowflake.generate();
        assert!(new_id > initial_id);
    }
}
