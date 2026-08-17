
/**
 * ========================== PressResponder Tutorial ==========================
 *
 * The `PressResponder` class helps you create press interactions by analyzing the
 * geometry of elements and observing when another responder (e.g. ScrollView)
 * has stolen the touch lock. It offers hooks for your component to provide
 * interaction feedback to the user:
 *
 * - When a press has activated (e.g. highlight an element)
 * - When a press has deactivated (e.g. un-highlight an element)
 * - When a press sould trigger an action, meaning it activated and deactivated while within the geometry of the element without the lock being stolen.
 *
 * A high quality interaction isn't as simple as you might think. There should
 * be a slight delay before activation. Moving your finger beyond an element's
 * bounds should trigger deactivation, but moving the same finger back within an
 * element's bounds should trigger reactivation.
 *
 * 1- In order to use `PressResponder`, do the following:
 *```js
 *     const pressResponder = new PressResponder(config);
 *```
 *   2.   Choose the rendered component who should collect the press events. On that
 *   element, spread `pressability.getEventHandlers()` into its props.
 *```js
 *    return (
 *      <View {...this.state.pressResponder.getEventHandlers()} />
 *    );
 *```
 * 3. Reset `PressResponder` when your component unmounts.
 *```js
 *    componentWillUnmount() {
 *      this.state.pressResponder.reset();
 *    }
 *```
 * ==================== Implementation Details ====================
 *
 * `PressResponder` only assumes that there exists a `HitRect` node. The `PressRect`
 * is an abstract box that is extended beyond the `HitRect`.
 *
 * # Geometry
 *  When the press is released outside the `HitRect`,
 *  the responder is NOT eligible for a "press".
 *
 */
