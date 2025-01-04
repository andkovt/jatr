use std::convert::Infallible;
use std::fmt;
use std::fmt::Write;
use std::marker::PhantomData;
use std::str::FromStr;
use serde::{de, Deserialize, Deserializer};
use serde::de::{EnumAccess, Error, MapAccess, Visitor};
use void::Void;
use crate::tasks::Action;

pub fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map or bool")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(FromStr::from_str(v.to_string().as_str()).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

pub fn string_or_struct2<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_enum<A>(self, data: A) -> Result<T, A::Error>
        where
            A: EnumAccess<'de>,
        {
            // println!("{:?}", data);
            Ok(FromStr::from_str("asd").unwrap())

            // Ok(T::from(Action::If {command: String::from("fale")}))
        }

        fn visit_map<M>(self, mut map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // let ff : Option<String> = map.next_value()?;

            // println!("{:?}", ff);
            println!("{:?}", std::any::type_name::<T>());

            // println!("{:?}", map.size_hint());

            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            let result = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map));

            match result {
                Ok(_) => { },
                Err(ref e) => {
                    println!("Error: {:?}", e);
                    // Err(e)
                }
            }

            result
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

pub fn string_or_struct3<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Infallible>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Infallible>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, mut map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // let ff : Option<String> = map.next_value()?;

            // println!("{:?}", ff);
            println!("{:?}", std::any::type_name::<T>());

            // println!("{:?}", map.size_hint());

            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            let result = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map));

            match result {
                Ok(_) => { },
                Err(ref e) => {
                    println!("Error: {:?}", e);
                    // Err(e)
                }
            }

            result
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}