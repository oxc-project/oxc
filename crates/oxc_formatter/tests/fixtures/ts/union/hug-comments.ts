// A single-line block comment right before the union is the first member's: no hug
type BlockRef = /* c */ CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX | null;
type BlockObject = /* c */ { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccccccccc: 1 } | null;
type TwoBlocks = /* a */ /* b */ CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXXXXXXXXXXXXXXX | null;
type BlockFits = /* c */ Foo | null;
function param(a: /* c */ { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cc: 1 } | null) {}
type TypeArguments = Foo</* c */ { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; c: 1 } | null>;
const annotation: /* c */ { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccccccc: 1 } | null = x;
const cast = x as /* c */ { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccccccccc: 1 } | null;

// Any other leading comment keeps the hug
type MultilineBlock = /* multi
  line */ CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXXXXXXX | null;
type BlockEndsLine = /* c */
  CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXXXXXXXXXXX | null;
type OwnLineRef =
  // c
  CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXXXXXXXXXXXXXXX | null;
type OwnLineBlock =
  /* c */
  CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXXXXXXXXXXXXXXX | null;
type OwnLineJsdoc =
  /** c */
  CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXX | null;
type OwnLineObject =
  // c
  { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccc: 1 } | null;
type OwnLineFits =
  // c
  Foo | null;

// A comment a parent prints first is not the first member's: the hug stays
type CheckPosition<T> = /* c */ { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccccccccccc: 1 } | null extends T ? A : B;
interface Predicate { isFoo(): this is /* c */ { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccccc: 1 } | null }
