/**
 * Measures the `HitRect` node on activation. The Bounding rectangle is with
 * respect to viewport - not page, so adding the `pageXOffset/pageYOffset`
 * should result in points that are in the same coordinate system as an
 * event's `globalX/globalY` data values.
 *
 * - Consider caching this for the lifetime of the component, or possibly being
 *   able to share this cache between any `ScrollMap` view.
 *
 * @private
 * @sideeffects
 */
