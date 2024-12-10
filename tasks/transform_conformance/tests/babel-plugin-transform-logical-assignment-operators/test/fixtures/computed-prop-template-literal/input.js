let boundObj;

boundObj[`abc`] &&= 1;
unboundObj[`abc`] &&= 2;

boundObj[`abc${foo}def`] &&= 3;
unboundObj[`abc${foo}def`] &&= 4;
