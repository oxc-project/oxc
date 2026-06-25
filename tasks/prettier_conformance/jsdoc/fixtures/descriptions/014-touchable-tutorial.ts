
/**
 * `Touchable`: Taps done right.
 *
 * You hook your `ResponderEventPlugin` events into `Touchable`. `Touchable`
 * will measure time/geometry and tells you when to give feedback to the user.
 *
 * ====================== Touchable Tutorial ===============================
 * The `Touchable` mixin helps you handle the "press" interaction. It analyzes
 *  the geometry of elements, and observes when another responder (scroll view
 * etc) has stolen the touch lock. It notifies your component when it should
 * give feedback to the user. (bouncing/highlighting/unhighlighting).
 *
 * - When a touch was activated (typically you highlight)
 * - When a touch was deactivated (typically you unhighlight)
 * - When a touch was "pressed" - a touch ended while still within the geometry
 *   of the element, and no other element (like scroller) has "stolen" touch
 *   lock ("responder") (Typically you bounce the element).
 *
 * A good tap interaction isn't as simple as you might think. There should be a
 * slight delay before showing a highlight when starting a touch. If a
 * subsequent touch move exceeds the boundary of the element, it should
 * unhighlight, but if that same touch is brought back within the boundary, it
 * should rehighlight again. A touch can move in and out of that boundary
 * several times, each time toggling highlighting, but a "press" is only
 * triggered if that touch ends while within the element's boundary and no
 * scroller (or anything else) has stolen the lock on touches.
 *
 * To create a new type of component that handles interaction using the
 * `Touchable` mixin, do the following:
 *
 * - Initialize the `Touchable` state.
 *```js
 *   getInitialState: function(   ) {
 *     return merge(this.touchableGetInitialState(), yourComponentState);
 *   }
 *```
 * - Choose the rendered component who's touches should start the interactive
 *   sequence. On that rendered node, forward all `Touchable` responder
 *   handlers. You can choose any rendered node you like. Choose a node whose
 *   hit target you'd like to instigate the interaction sequence:
 *```js
 *   // In render function:
 *   return (
 *     <View
 *
 *       onStartShouldSetResponder={this.touchableHandleStartShouldSetResponder}
 *       onResponderTerminationRequest={this.touchableHandleResponderTerminationRequest}
 *       onResponderGrant={this.touchableHandleResponderGrant}
 *       onResponderMove={this.touchableHandleResponderMove}
 *       onResponderRelease={this.touchableHandleResponderRelease}
 *       onResponderTerminate={this.touchableHandleResponderTerminate}>
 *       <View>
 *         Even though the hit detection/interactions are triggered by the
 *         wrapping (typically larger) node, we usually end up implementing
 *         custom logic that highlights this inner one.
 *       </View>
 *     </View>
 *   );
 *```
 * - You may set up your own handlers for each of these events, so long as you
 *   also invoke the `touchable*` handlers inside of your custom handler.
 *
 * - Implement the handlers on your component class in order to provide
 *   feedback to the user. See documentation for each of these class methods
 *   that you should implement.
 *```js
 *   touchableHandlePress: function() {
 *      this.performBounceAnimation();  // or whatever you want to do.
 *   },
 *   touchableHandleActivePressIn: function() {
 *     this.beginHighlighting(...);  // Whatever you like to convey activation
 *   },
 *   touchableHandleActivePressOut: function() {
 *     this.endHighlighting(...);  // Whatever you like to convey deactivation
 *   },
 *```
 * - There are more advanced methods you can implement (see documentation below):
 * ```js
 *   touchableGetHighlightDelayMS: function() {
 *     return 20;
 *   }
 *   // In practice, *always* use a predeclared constant (conserve memory).
 *   touchableGetPressRectOffset: function() {
 *     return {top: 20, left: 20, right: 20, bottom: 100};
 *   }
 * ```
 */
