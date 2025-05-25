use crate::attribute::{Attribute, AttributeName, AttributeCommon};
use crate::character::{CharacterConfig, CharacterName, CharacterStaticData};
use crate::character::character_common_data::CharacterCommonData;
use crate::character::character_sub_stat::CharacterSubStatFamily;
use crate::character::characters::clorinde::ClorindeDamageEnum;
use crate::character::characters::cryo::escoffier::EscoffierDamageEnum::E1;
use crate::character::characters::kamisato_ayaka::KamisatoAyakaEffect;
use crate::character::macros::{damage_enum, damage_ratio, skill_map, skill_type};
use crate::character::skill_config::CharacterSkillConfig;
use crate::character::traits::{CharacterSkillMap, CharacterSkillMapItem, CharacterTrait};
use crate::common::{ChangeAttribute, Element, SkillType, StatName, WeaponType};
use crate::common::i18n::{locale, hit_n_dmg, plunging_dmg, charged_dmg};
use crate::common::item_config_type::{ItemConfig, ItemConfigType};
use crate::damage::damage_builder::DamageBuilder;
use crate::damage::DamageContext;
use crate::target_functions::TargetFunction;
use crate::team::TeamQuantization;
use crate::weapon::weapon_common_data::WeaponCommonData;

pub struct EscoffierSkillType {
    pub normal_dmg1: [f64; 15],
    pub normal_dmg2: [f64; 15],
    pub normal_dmg3: [f64; 15],
    pub charged_dmg1: [f64; 15],
    pub plunging_dmg1: [f64; 15],
    pub plunging_dmg2: [f64; 15],
    pub plunging_dmg3: [f64; 15],
    pub e_dmg1: [f64; 15],
    pub e_dmg2: [f64; 15],
    pub q_dmg1: [f64; 15],
}

pub const ESCOFFIER_SKILL: EscoffierSkillType = EscoffierSkillType {
    normal_dmg1: [0.51551, 0.55747, 0.59943, 0.659373, 0.701333, 0.749287, 0.815225, 0.881162, 0.947099, 1.019031, 1.090963, 1.162894, 1.234826, 1.306757, 1.378689],
    normal_dmg2: [0.475933, 0.514671, 0.55341, 0.608751, 0.64749, 0.691762, 0.752638, 0.813513, 0.874388, 0.940797, 1.007206, 1.073615, 1.140025, 1.206434, 1.272843],
    normal_dmg3: [0.6439, 0.6963, 0.7487, 0.8236, 0.876, 0.9359, 1.0182, 1.1006, 1.1829, 1.2728, 1.3626, 1.4525, 1.5423, 1.6322, 1.722],
    charged_dmg1: [1.15412, 1.24806, 1.342, 1.4762, 1.57014, 1.6775, 1.82512, 1.97274, 2.12036, 2.2814, 2.44244, 2.60348, 2.76452, 2.92556, 3.0866],
    plunging_dmg1: [0.6393, 0.6914, 0.7434, 0.8177, 0.8698, 0.9293, 1.011, 1.0928, 1.1746, 1.2638, 1.353, 1.4422, 1.5314, 1.6206, 1.7098],
    plunging_dmg2: [1.2784, 1.3824, 1.4865, 1.6351, 1.7392, 1.8581, 2.0216, 2.1851, 2.3486, 2.527, 2.7054, 2.8838, 3.0622, 3.2405, 3.4189],
    plunging_dmg3: [1.5968, 1.7267, 1.8567, 2.0424, 2.1723, 2.3209, 2.5251, 2.7293, 2.9336, 3.1564, 3.3792, 3.602, 3.8248, 4.0476, 4.2704],
    e_dmg1: [0.504, 0.5418, 0.5796, 0.63, 0.6678, 0.7056, 0.756, 0.8064, 0.8568, 0.9072, 0.9576, 1.008, 1.071, 1.134, 1.197],
    e_dmg2: [1.2, 1.29, 1.38, 1.5, 1.59, 1.68, 1.8, 1.92, 2.04, 2.16, 2.28, 2.4, 2.55, 2.7, 2.85],
    q_dmg1: [5.928, 6.3726, 6.8172, 7.41, 7.8546, 8.2992, 8.892, 9.4848, 10.0776, 10.6704, 11.2632, 11.856, 12.597, 13.338, 14.079]
};

damage_enum!(
    EscoffierDamageEnum
    Normal1
    Normal2
    Normal3
    Charged1
    Plunging1
    Plunging2
    Plunging3
    E1
    E2
    Q1
);

impl EscoffierDamageEnum {
    pub fn get_skill_type(&self) -> SkillType {
        use EscoffierDamageEnum::*;
        match *self {
            Normal1 | Normal2 | Normal3 => SkillType::NormalAttack,
            Charged1 => SkillType::ChargedAttack,
            Plunging1 => SkillType::PlungingAttackInAction,
            Plunging2 | Plunging3 => SkillType::PlungingAttackOnGround,
            E1 => SkillType::ElementalSkill,
            E2 => SkillType::ElementalSkill,
            Q1 => SkillType::ElementalBurst
        }
    }

    pub fn get_element(&self) -> Element {
        use EscoffierDamageEnum::*;
        match *self {
            Normal1 | Normal2 | Normal3 | Charged1 | Plunging1 | Plunging2 | Plunging3 => Element::Physical,
            E1 | E2 | Q1 => Element::Electro
        }
    }
}

struct EscoffierEffect {
    pub talent1_rate: f64,
    pub has_talent1: bool
}

impl<A: Attribute> ChangeAttribute<A> for EscoffierEffect {
    fn change_attribute(&self, attribute: &mut A) {
        if self.has_talent1 {
            attribute.add_atk_percentage("无天赋", 0f64);
        }
    }
}

pub struct Escoffier;

impl CharacterTrait for Escoffier {
    const STATIC_DATA: CharacterStaticData = CharacterStaticData {
        name: CharacterName::Escoffier,
        internal_name: "Escoffier",
        name_locale: locale!(
            zh_cn: "爱可菲",
            en: "Escoffier"
        ),
        element: Element::Electro,
        hp: [1039, 2695, 3586, 5366, 5999, 6902, 7747, 8659, 9292, 10213, 10846, 11777, 12410, 13348],
        atk: [27, 70, 93, 139, 156, 179, 201, 225, 241, 265, 282, 306, 322, 347],
        def: [57, 148, 197, 294, 329, 378, 425, 475, 509, 560, 594, 646, 680, 732],
        sub_stat: CharacterSubStatFamily::CriticalRate192,
        weapon_type: WeaponType::Polearm,
        star: 5,
        skill_name1: locale!(
            zh_cn: "后厨手艺",
            en: "Kitchen Skills"
        ),
        skill_name2: locale!(
            zh_cn: "低温烹饪",
            en: "Low-Temperature Cooking"
        ),
        skill_name3: locale!(
            zh_cn: "花刀技法",
            en: "Scoring Cuts"
        ),
    };
    type SkillType = EscoffierSkillType;
    const SKILL: Self::SkillType = ESCOFFIER_SKILL;
    type DamageEnumType = EscoffierDamageEnum;
    type RoleEnum = ();

    #[cfg(not(target_family = "wasm"))]
    const SKILL_MAP: CharacterSkillMap = CharacterSkillMap {
        skill1: skill_map!(
            EscoffierDamageEnum
            Normal1 hit_n_dmg!(1)
            Normal2 hit_n_dmg!(2)
            Normal3 hit_n_dmg!(3)
            Charged1 charged_dmg!()
            Plunging1 plunging_dmg!(1)
            Plunging2 plunging_dmg!(2)
            Plunging3 plunging_dmg!(3)
        ),
        skill2: skill_map!(
            EscoffierDamageEnum
            E1 locale!(zh_cn: "技能伤害", en: "Skill DMG")
            E2 locale!(zh_cn: "冻霜芭菲伤害", en: "Skill DMG")
        ),
        skill3: skill_map!(
            EscoffierDamageEnum
            Q1 locale!(zh_cn: "技能伤害", en: "Skill DMG")
        ),
    };
    
    fn damage_internal<D: DamageBuilder>(context: &DamageContext<'_, D::AttributeType>, s: usize, config: &CharacterSkillConfig, fumo: Option<Element>) -> D::Result {
        let s: EscoffierDamageEnum = num::FromPrimitive::from_usize(s).unwrap();
        let (s1, s2, s3) = context.character_common_data.get_3_skill();

        use EscoffierDamageEnum::*;
        let mut builder = D::new();

        let ratio = match s {
            Normal1 => ESCOFFIER_SKILL.normal_dmg1[s1],
            Normal2 => ESCOFFIER_SKILL.normal_dmg2[s1],
            Normal3 => ESCOFFIER_SKILL.normal_dmg3[s1],
            Charged1 => ESCOFFIER_SKILL.charged_dmg1[s1],
            Plunging1 => ESCOFFIER_SKILL.plunging_dmg1[s1],
            Plunging2 => ESCOFFIER_SKILL.plunging_dmg2[s1],
            Plunging3 => ESCOFFIER_SKILL.plunging_dmg3[s1],
            E1 => ESCOFFIER_SKILL.e_dmg1[s2],
            E2 => ESCOFFIER_SKILL.e_dmg2[s2],
            Q1 => ESCOFFIER_SKILL.q_dmg1[s3],
        };
        builder.add_atk_ratio("技能倍率", ratio);

        builder.damage(
            &context.attribute,
            &context.enemy,
            s.get_element(),
            s.get_skill_type(),
            context.character_common_data.level,
            fumo
        )
    }

    fn new_effect<A: Attribute>(common_data: &CharacterCommonData, config: &CharacterConfig) -> Option<Box<dyn ChangeAttribute<A>>> {
        match *config {
            CharacterConfig::Escoffier { talent1_rate } => Some(Box::new(EscoffierEffect {
                talent1_rate,
                has_talent1: common_data.has_talent1
            })),
            _ => None
        }
    }
    
    fn get_target_function_by_role(role_index: usize, team: &TeamQuantization, c: &CharacterCommonData, w: &WeaponCommonData) -> Box<dyn TargetFunction> {
        unimplemented!()
    }
}
