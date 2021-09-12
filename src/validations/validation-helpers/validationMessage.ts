import { ValidationResult } from 'joi';

const validationMessage = (validationResult: ValidationResult): Object | null => {
  if (validationResult.error) {
    const errors = validationResult.error?.details
      .map((o) => {
        if (o.context?.key) return { [o.context?.key]: o.message };
        return {};
      })
      .reduce((r, c) => Object.assign(r, c), {});

    return { errors };
  }

  return null;
};

export default validationMessage;
