import styled from '@xstyled/styled-components'
import unstyled from '@xstyled/styled-components-test'

const Test = styled.div`
  width: 100%;
`
const Test2 = true ? styled.div`` : styled.div``
const styles = { One: styled.div`` }
let Component
Component = styled.div``
const WrappedComponent = styled(Inner)``
const NoTransformComponent = unstyled.div``;
