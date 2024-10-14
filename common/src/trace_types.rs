use std::hash::Hash;

use revm::primitives::{ alloy_primitives::utils::{ ParseUnits, Unit }, FixedBytes };
use serde::{ Deserialize, Serialize };
use tiny_keccak::Hasher;

macro_rules! solidity_types {
    ($($type_name:ident => $type_value:expr),*) => {
        // Constants with canonical type names
        $(
            #[allow(unused)]
            const $type_name: &str = $type_value;
        )*

        // Enum with variant names matching the constants
        #[derive(Debug, PartialEq, Clone)]
        pub enum TypeLabel {
            $(
                $type_name,
            )*
            /// either a mapping, struct, or array types
            Other(String)
        }

        // Implementation for matching a &str to an enum variant
        impl TypeLabel {
            pub fn try_from_str(s: &str) -> Option<TypeLabel> {
                match s {
                    $(
                        $type_value => Some(TypeLabel::$type_name),
                    )*
                        _ => Some(TypeLabel::Other(s.to_string())),
                }
            }

            pub fn to_str(&self) -> &str {
                match self {
                    $(
                        TypeLabel::$type_name => $type_value,
                    )*
                    TypeLabel::Other(s) => s,
                }
            }
        }

        impl<'de> Deserialize<'de> for TypeLabel {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
                let s: String = Deserialize::deserialize(deserializer)?;

                match s.as_str() {
                    $(
                        $type_value => Ok(TypeLabel::$type_name),
                    )*
                    _ => Ok(TypeLabel::Other(s)),
                }
            }
        }
    };
}

// Generate constants and enum from the Solidity types
solidity_types!(
    UINT => "uint",
    INT => "int",
    ADDRESS => "address",
    BOOL => "bool",
    BYTES => "bytes",
    STRING => "string",
    FIXED => "fixed",
    UFIXED => "ufixed",
    
    UINT8 => "uint8",
    UINT16 => "uint16",
    UINT24 => "uint24",
    UINT32 => "uint32",
    UINT40 => "uint40",
    UINT48 => "uint48",
    UINT56 => "uint56",
    UINT64 => "uint64",
    UINT72 => "uint72",
    UINT80 => "uint80",
    UINT88 => "uint88",
    UINT96 => "uint96",
    UINT104 => "uint104",
    UINT112 => "uint112",
    UINT120 => "uint120",
    UINT128 => "uint128",
    UINT136 => "uint136",
    UINT144 => "uint144",
    UINT152 => "uint152",
    UINT160 => "uint160",
    UINT168 => "uint168",
    UINT176 => "uint176",
    UINT184 => "uint184",
    UINT192 => "uint192",
    UINT200 => "uint200",
    UINT208 => "uint208",
    UINT216 => "uint216",
    UINT224 => "uint224",
    UINT232 => "uint232",
    UINT240 => "uint240",
    UINT248 => "uint248",
    UINT256 => "uint256",

    INT8 => "int8",
    INT16 => "int16",
    INT24 => "int24",
    INT32 => "int32",
    INT40 => "int40",
    INT48 => "int48",
    INT56 => "int56",
    INT64 => "int64",
    INT72 => "int72",
    INT80 => "int80",
    INT88 => "int88",
    INT96 => "int96",
    INT104 => "int104",
    INT112 => "int112",
    INT120 => "int120",
    INT128 => "int128",
    INT136 => "int136",
    INT144 => "int144",
    INT152 => "int152",
    INT160 => "int160",
    INT168 => "int168",
    INT176 => "int176",
    INT184 => "int184",
    INT192 => "int192",
    INT200 => "int200",
    INT208 => "int208",
    INT216 => "int216",
    INT224 => "int224",
    INT232 => "int232",
    INT240 => "int240",
    INT248 => "int248",
    INT256 => "int256",

    FIXEDMXN => "fixedMxN",
    UFIXEDMXN => "ufixedMxN",

    BYTES1 => "bytes1",
    BYTES2 => "bytes2",
    BYTES3 => "bytes3",
    BYTES4 => "bytes4",
    BYTES5 => "bytes5",
    BYTES6 => "bytes6",
    BYTES7 => "bytes7",
    BYTES8 => "bytes8",
    BYTES9 => "bytes9",
    BYTES10 => "bytes10",
    BYTES11 => "bytes11",
    BYTES12 => "bytes12",
    BYTES13 => "bytes13",
    BYTES14 => "bytes14",
    BYTES15 => "bytes15",
    BYTES16 => "bytes16",
    BYTES17 => "bytes17",
    BYTES18 => "bytes18",
    BYTES19 => "bytes19",
    BYTES20 => "bytes20",
    BYTES21 => "bytes21",
    BYTES22 => "bytes22",
    BYTES23 => "bytes23",
    BYTES24 => "bytes24",
    BYTES25 => "bytes25",
    BYTES26 => "bytes26",
    BYTES27 => "bytes27",
    BYTES28 => "bytes28",
    BYTES29 => "bytes29",
    BYTES30 => "bytes30",
    BYTES31 => "bytes31",
    BYTES32 => "bytes32"
);

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct StorageItem<Type = String> {
    #[serde(rename = "astId")]
    pub ast_id: u64,
    pub contract: String,
    pub label: String,
    pub offset: u8,
    pub slot: String,
    // TODO: this should be properly handled
    #[serde(rename = "type")]
    pub types: Type,
}

impl StorageItem<String> {
    pub fn to_typed(&self, types: StorageTypes) -> StorageItem<StorageTypes> {
        StorageItem {
            ast_id: self.ast_id,
            contract: self.contract.clone(),
            label: self.label.clone(),
            offset: self.offset,
            slot: self.slot.clone(),
            types,
        }
    }
}

pub trait Stored {
    fn as_storage_keys(&self) -> [u8; 32];
}

impl Stored for StorageItem<StorageTypes> {
    fn as_storage_keys(&self) -> [u8; 32] {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Encoding {
    Inplace,
    Mapping,
    DynamicArray,
    Bytes,
}

impl<'de> Deserialize<'de> for Encoding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s: String = Deserialize::deserialize(deserializer)?;

        match s.as_str() {
            "inplace" => Ok(Encoding::Inplace),
            "mapping" => Ok(Encoding::Mapping),
            "dynamicArray" => Ok(Encoding::DynamicArray),
            "bytes" => Ok(Encoding::Bytes),
            _ => Err(serde::de::Error::custom("Invalid encoding type")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericStorageTypes {
    pub encoding: Encoding,
    pub number_of_bytes: String,
    members: Option<Vec<StorageItem<String>>>,
    key: Option<String>,
    value: Option<String>,
    base: Option<String>,
    pub label: TypeLabel,
}

impl GenericStorageTypes {
    pub fn from_values(values: serde_json::Value) -> Vec<(String, Self)> {
        let values = values["types"].clone();

        values
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| { (k.clone(), serde_json::from_value(v.clone()).unwrap()) })
            .collect()
    }
}

#[cfg(test)]
mod parse_test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_decode_struct() {
        let val =
            json!({
            "types": {
                "t_struct(S)13_storage": {
      "encoding": "inplace",
      "label": "struct A.S",
      "members": [
        {
          "astId": 3,
          "contract": "fileA:A",
          "label": "a",
          "offset": 0,
          "slot": "0",
          "type": "t_uint128"
        },
        {
          "astId": 5,
          "contract": "fileA:A",
          "label": "b",
          "offset": 16,
          "slot": "0",
          "type": "t_uint128"
        },
        {
          "astId": 9,
          "contract": "fileA:A",
          "label": "staticArray",
          "offset": 0,
          "slot": "1",
          "type": "t_array(t_uint256)2_storage"
        },
        {
          "astId": 12,
          "contract": "fileA:A",
          "label": "dynArray",
          "offset": 0,
          "slot": "3",
          "type": "t_array(t_uint256)dyn_storage"
        }
      ],
      "numberOfBytes": "128"
    }
            }
        });

        let types = GenericStorageTypes::from_values(val);
        let (k, v) = types[0].clone();

        assert_eq!(k, "t_struct(S)13_storage");
        assert_eq!(v.encoding, Encoding::Inplace);
        assert_eq!(v.number_of_bytes.as_str(), "128");
        assert_eq!(v.label, TypeLabel::Other("struct A.S".to_string()));
        assert_eq!(v.members.unwrap().len(), 4);
    }

    #[test]
    fn test_decode_mapping() {
        let val =
            json!(
            {
                "types": {
                    "t_mapping(t_uint256,t_uint256)14_storage": {
                        "encoding": "mapping",
                        "key": "t_uint256",
                        "label": "mapping(uint256 => uint256)",
                        "numberOfBytes": "64",
                        "value": "t_uint256"
                    }
                }
            }
        );

        let types = GenericStorageTypes::from_values(val);
        let (k, v) = types[0].clone();

        assert_eq!(k, "t_mapping(t_uint256,t_uint256)14_storage");
        assert_eq!(v.encoding, Encoding::Mapping);
        assert_eq!(v.number_of_bytes.as_str(), "64");
        assert_eq!(v.label, TypeLabel::Other("mapping(uint256 => uint256)".to_string()));
        assert_eq!(v.key.unwrap(), "t_uint256");
    }

    #[test]
    fn test_decode_dynamic_array() {
        let val =
            json!(
            {
                "types": {
                    "t_array(t_uint256)15_storage": {
                        "encoding": "dynamicArray",
                        "label": "uint256[]",
                        "numberOfBytes": "64",
                        "base": "t_uint256"
                    }
                }
            }
        );

        let types = GenericStorageTypes::from_values(val);
        let (k, v) = types[0].clone();

        assert_eq!(k, "t_array(t_uint256)15_storage");
        assert_eq!(v.encoding, Encoding::DynamicArray);
        assert_eq!(v.number_of_bytes.as_str(), "64");
        assert_eq!(v.label, TypeLabel::Other("uint256[]".to_string()));
        assert_eq!(v.base.unwrap(), "t_uint256");
    }

    #[test]
    fn test_decode_native() {
        let val =
            json!(
            {
                "types": {
                    "t_uint256": {
                        "encoding": "inplace",
                        "label": "uint256",
                        "numberOfBytes": "32"
                    }
                }
            }
        );

        let types = GenericStorageTypes::from_values(val);
        let (k, v) = types[0].clone();

        assert_eq!(k, "t_uint256");
        assert_eq!(v.encoding, Encoding::Inplace);
        assert_eq!(v.number_of_bytes.as_str(), "32");
        assert_eq!(v.label, TypeLabel::UINT256);
    }

    #[test]
    fn test_decode_multiple() {
        let val =
            json!(
            {
                "types": {
                    "t_uint256": {
                        "encoding": "inplace",
                        "label": "uint256",
                        "numberOfBytes": "32"
                    },
                    "t_array(t_uint256)15_storage": {
                        "encoding": "dynamicArray",
                        "label": "uint256[]",
                        "numberOfBytes": "64",
                        "base": "t_uint256"
                    },
                    "t_mapping(t_uint256,t_uint256)14_storage": {
                        "encoding": "mapping",
                        "key": "t_uint256",
                        "label": "mapping(uint256 => uint256)",
                        "numberOfBytes": "64",
                        "value": "t_uint256"
                    },
                    "t_struct(S)13_storage": {
                        "encoding": "inplace",
                        "label": "struct A.S",
                        "members": [
                            {
                                "astId": 3,
                                "contract": "fileA:A",
                                "label": "a",
                                "offset": 0,
                                "slot": "0",
                                "type": "t_uint128"
                            },
                            {
                                "astId": 5,
                                "contract": "fileA:A",
                                "label": "b",
                                "offset": 16,
                                "slot": "0",
                                "type": "t_uint128"
                            },
                            {
                                "astId": 9,
                                "contract": "fileA:A",
                                "label": "staticArray",
                                "offset": 0,
                                "slot": "1",
                                "type": "t_array(t_uint256)2_storage"
                            },
                            {
                                "astId": 12,
                                "contract": "fileA:A",
                                "label": "dynArray",
                                "offset": 0,
                                "slot": "3",
                                "type": "t_array(t_uint256)dyn_storage"
                            }
                        ],
                        "numberOfBytes": "128"
                    }
                }
            }
        );

        let types = GenericStorageTypes::from_values(val);

        assert_eq!(types.len(), 4);
    }
}

impl GenericStorageTypes {
    pub fn try_as_dynamic_array(&self) -> Option<DynamicArrayStorageItemType> {
        match self.encoding {
            Encoding::DynamicArray =>
                Some(DynamicArrayStorageItemType {
                    encoding: self.encoding.clone(),
                    number_of_bytes: self.number_of_bytes.parse().expect("Expected a number"),
                    base: self.base.clone()?,
                    label: self.label.clone(),
                }),
            _ => None,
        }
    }

    pub fn try_as_mapping(&self) -> Option<MappingStorageItemType> {
        match self.encoding {
            Encoding::Mapping =>
                Some(MappingStorageItemType {
                    encoding: self.encoding.clone(),
                    number_of_bytes: self.number_of_bytes.parse().expect("Expected a number"),
                    key: self.key.clone().unwrap(),
                    value: self.value.clone().unwrap(),
                    label: self.label.clone(),
                }),
            _ => None,
        }
    }

    pub fn try_as_struct(&self) -> Option<StructStorageItemType> {
        match self.encoding {
            Encoding::Inplace =>
                Some(StructStorageItemType {
                    encoding: self.encoding.clone(),
                    number_of_bytes: self.number_of_bytes.parse().expect("Expected a number"),
                    members: self.members.clone().expect("Expected members"),
                    label: self.label.clone(),
                }),
            _ => None,
        }
    }

    pub fn try_as_primitive(&self) -> Option<PrimitiveStorageItemType> {
        // make sure it's not a struct
        if let Some(_) = self.try_as_struct() {
            return None;
        }

        match self.encoding {
            Encoding::Inplace =>
                Some(PrimitiveStorageItemType {
                    encoding: self.encoding.clone(),
                    number_of_bytes: self.number_of_bytes.parse().expect("Expected a number"),
                    label: self.label.clone(),
                }),
            _ => None,
        }
    }
}

pub enum StorageTypes {
    DynamicArray(DynamicArrayStorageItemType),
    Mapping(MappingStorageItemType),
    Struct(StructStorageItemType),
    Primitive(PrimitiveStorageItemType),
}

impl StorageTypes {
    pub fn from_generic_storage_type(generic_storage_type: GenericStorageTypes) -> Self {
        match generic_storage_type.encoding {
            Encoding::Inplace => {
                if let Some(struct_storage_item) = generic_storage_type.try_as_struct() {
                    StorageTypes::Struct(struct_storage_item)
                } else {
                    let primitive_storage_item = generic_storage_type
                        .try_as_primitive()
                        .expect("Can't decode as primitive or struct");

                    StorageTypes::Primitive(primitive_storage_item)
                }
            }
            Encoding::Mapping => todo!("Mapping encoding not supported for now"),
            Encoding::DynamicArray => todo!("Dynamic array encoding not supported for now"),
            Encoding::Bytes => todo!("Bytes encoding not supported for now"),
        }
    }
}

// TODO : make a simple parser for type generating storage keys for types

#[derive(Debug, Clone)]
pub struct DynamicArrayStorageItemType {
    pub encoding: Encoding,
    pub number_of_bytes: u64,
    // type key
    pub base: String,
    pub label: TypeLabel,
}

#[derive(Debug, Clone)]
pub struct MappingStorageItemType {
    pub encoding: Encoding,
    pub number_of_bytes: u64,
    // type key
    pub key: String,
    // type key
    pub value: String,
    pub label: TypeLabel,
}

#[derive(Debug, Clone)]
pub struct StructStorageItemType {
    pub encoding: Encoding,
    pub number_of_bytes: u8,
    pub members: Vec<StorageItem<String>>,
    pub label: TypeLabel,
}

#[derive(Debug, Clone)]
pub struct PrimitiveStorageItemType {
    pub encoding: Encoding,
    pub number_of_bytes: u8,
    pub label: TypeLabel,
}

impl PrimitiveStorageItemType {
    pub fn generate_storage_key(&self, slot: String) -> FixedBytes<32> {
        let uint = ParseUnits::parse_units(&slot, Unit::WEI).unwrap();

        revm::primitives::keccak256(uint.get_absolute().to_be_bytes::<32>())
    }
}
