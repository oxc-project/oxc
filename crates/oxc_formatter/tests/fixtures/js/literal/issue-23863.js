// Combining marks must count toward printWidth, matching Prettier (oxc-project/oxc#23863).
// The Tibetan subjoined letter U+0FB3 has zero `unicode-width` but Prettier counts it as 1,
// so this property value is > 80 columns and the assignment breaks after `monthsShort:`.
moment.defineLocale("bo", {
  monthsShort: "ཟླ་1_ཟླ་2_ཟླ་3_ཟླ་4_ཟླ་5_ཟླ་6_ཟླ་7_ཟླ་8_ཟླ་9_ཟླ་10_ཟླ་11_ཟླ་12".split("_"),
});
