
  import React, { memo } from "react";
  import { Text, View, StyleSheet } from "react-native";
  import * as d3Scale from "d3-scale";
  import * as array from "d3-array";
  import Svg, { G, Text as SVGText } from "react-native-svg";
  import { useLayout, useInlineStyle } from "./hooks";

  /**
   * @typedef {object} XAxisProps
   * @property {number} [spacingOuter]
   * Spacing between the labels. Only applicable if
   * `scale=d3Scale.scaleBand` and should then be equal to `spacingOuter` prop on the
   * actual BarChart
   *
   * Default is `0.05`
   * @property {number} [spacingInner] Spacing between the labels. Only applicable if
   * `scale=d3Scale.scaleBand` and should then be equal to `spacingInner` prop on the
   * actual BarChart
   *
   * Default is `0.05`
   * @property {d3Scale.scaleLinear} [scale] Should be the same as passed into the charts `xScale`
   * Default is `d3Scale.scaleLinear`
   *
   * @property {()=>any} [xAccessor] Default is `({index}) => index`
   * @property {number} [max]
   * @property {number} [min]
   */

  /**
   * @type {React.FC<XAxisProps & import("react-native-svg").TextProps>}
   */
  const XAxis = memo(
    ({
      contentInset: { left = 0, right = 0 } = {},
      style,
      data,
      numberOfTicks,
      children,
      min,
      max,
      spacingInner = 0.05,
      spacingOuter = 0.05,
      xAccessor = ({ index }) => index,
      scale = d3Scale.scaleLinear,
      formatLabel = value => value,
      ...svg
    })=>{})
