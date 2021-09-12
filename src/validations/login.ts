import Joi, { Schema } from 'joi';
import validationMessage from './validation-helpers/validationMessage';

const loginValidation = (data: Object) => {
  const schema: Schema = Joi.object({
    email: Joi.string().email().min(6).max(255).required(),
    password: Joi.string().min(6).required(),
  });

  const validation = schema.validate(data, { abortEarly: false });
  return validationMessage(validation);
};

export default loginValidation;
