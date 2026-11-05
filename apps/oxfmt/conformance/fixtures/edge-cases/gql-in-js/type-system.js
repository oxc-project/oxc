// Stable-spec type-system definitions in embedded position,
// exercising the breadth of the GraphQL formatter.
export const typeDefs = gql`
  "Schema description"
  schema    {
    query: Query
    mutation:    Mutation
  }

  """
  Block string
  description
  """
  type Query implements Node   &   Timestamped {
    node( id : ID! ) : Node
    items( first : Int = 10 , filter : ItemFilter ) : [Item!]!   @deprecated( reason : "use nodes" )
  }

  interface Node {
    id: ID!
  }

  union SearchResult =   User   |   Item

  enum Color {
    RED
    GREEN

    "with description"
    BLUE   @deprecated
  }

  input ItemFilter {
    name : String =   "default"
    tags : [ String! ]   =   [ "a" , "b" ]
    nested : ItemFilter
  }

  directive   @cacheControl ( maxAge : Int ) on FIELD_DEFINITION   |   OBJECT

  extend type Query {
    extra : String
  }

  scalar DateTime   @specifiedBy( url : "https://example.com" )
`;

// Directives after broken-out argument lists: `@skipAuth` stays glued to the
// return type or lands on its own line, never with a trailing space.
export const mutations = gql`
  type Mutation {
    create__TYPE_NAME__(input: Create__TYPE_NAME__Input!): __TYPE_NAME__!
      @skipAuth
    update__TYPE_NAME__(
      id: Int!
      input: Update__TYPE_NAME__Input!
    ): __TYPE_NAME__! @skipAuth
    delete__TYPE_NAME__(id: Int!): __TYPE_NAME__! @skipAuth
  }
`;

export const ops = gql`
  query Search( $term : String! , $color : Color = RED )   @cached {
    search( term : $term , filter : { tags : [ "x" ] , nested : { name : "y" } } ) {
      ... on User {  name  }
      ...itemFields    @include( if : true )
    }
  }

  subscription   OnChange {
    changed {   id   }
  }

  fragment itemFields on Item {
    id
    name
  }
`;
