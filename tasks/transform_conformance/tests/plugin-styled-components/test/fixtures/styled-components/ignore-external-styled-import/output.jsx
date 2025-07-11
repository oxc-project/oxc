import { styled } from '@material/ui';
import s from 'styled-components';
const Paragraph = s.p.withConfig({
  displayName: "input__Paragraph",
  componentId: "sc-g689do-0"
})(["color:green;"]);
const Foo = p => <Paragraph {...p} />;
const TestNormal = styled(Foo)({
  color: red
});
