// https://github.com/oxc-project/oxc/issues/21185
// Arrow function parameter in .map() callback placed on different line than Prettier
const updatePlanStartDateInCache = (planId: string, newStartDate: string) => {
  queryClient.setQueryData(
    queryKey,
    (cachedData: ReservesEmployerQueryQuery | undefined) => {
      if (!cachedData?.administrator?.employer?.plans) return cachedData;

      const updatedPlans = cachedData.administrator.employer.plans.map(
        plan =>
          plan.id === planId ? { ...plan, startDate: newStartDate } : plan
      );

      return {
        ...cachedData,
        administrator: {
          ...cachedData.administrator,
          employer: {
            ...cachedData.administrator.employer,
            plans: updatedPlans,
          },
        },
      };
    }
  );
};

// Just-under-boundary regression guard for the fix above.
// At printWidth=80, `(plan) =>` lands at col 79 — middle variant must
// still be selected (i.e. the line should hug, not break after `.map(`).
const updateOnePlanStartDate = (planId: string, newStartDate: string) => {
  queryClient.setQueryData(
    queryKey,
    (cachedData: ReservesEmployerQueryQuery | undefined) => {
      if (!cachedData?.administrator?.employer?.plans) return cachedData;

      const updatedPlan = cachedData.administrator.employer.plans.map((plan) =>
        plan.id === planId ? { ...plan, startDate: newStartDate } : plan
      );

      return updatedPlan;
    }
  );
};
