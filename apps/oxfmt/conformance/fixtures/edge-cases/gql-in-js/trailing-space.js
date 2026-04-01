export const schema = gql`
  type Mutation {
    create__TYPE_NAME__(input: Create__TYPE_NAME__Input!): __TYPE_NAME__!
      @skipAuth
    update__TYPE_NAME__(
      id: Int!
      input: Update__TYPE_NAME__Input!
    ): __TYPE_NAME__! @skipAuth
    delete__TYPE_NAME__(id: Int!): __TYPE_NAME__! @skipAuth
  }
`
