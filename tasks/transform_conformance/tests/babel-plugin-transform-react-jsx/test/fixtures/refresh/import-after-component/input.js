import { observer } from "mobx-react-lite";
import { useFoo } from "./useFoo";

export const BazComponent = observer(function BazComponent_() {
	const foo = useFoo();
	return (
		<>
			{foo}
			{bar}
		</>
	);
});

import { bar } from "./bar";
