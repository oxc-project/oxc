// https://github.com/oxc-project/oxc/issues/29222
// A hugged union has no break of its own: break after `=` instead of overflowing
export type ProjectDataRes = CrmOnlineRecordApiTypes.ProjectDataWithServices | null;
type CustomerProjectDetailsRes = CrmOnlineRecordApiTypes.CustomerProjectDetailsResponse | void;
type Generic = CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXX<string> | null;
type ThreeMembers = CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXX | null | void;
type TrailingComment = CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXX | null; // comment
// The type parameters stay on the `=` line
type WithTypeParams<TTTTTTTTTTTTTTTTTT> = CrmOnlineRecordApiTypes.CustomerProjectDetailsRes<TTTTTTTTTTTTTTTTTT> | null;
declare namespace NS {
  namespace Inner {
    export type Nested = CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXX | null;
  }
}

// A single member hugs too
type SingleMemberAliasWithALongName = | CrmOnlineRecordApiTypes.CustomerProjectDetailsResponseXXXXXXXXXXXXXXXXXXXXXXXXXXX;

// An object-like member still breaks inside, hugging the `=`
type A = { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccc: boolean } | null;
type C = Array<{ aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; ccccccccccc: boolean }> | null;
type ObjectFitsOnNextLine = { aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbb: number } | null;
type TypeArguments = Foo<aaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccccccccccccc> | null;
