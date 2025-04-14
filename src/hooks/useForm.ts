import { useState, ChangeEvent, FormEvent } from 'react';

type ValidationRules<T> = {
  [K in keyof T]?: (value: T[K]) => string | null;
};

export type FormErrors<T> = {
  [K in keyof T]?: string;
};

/**
 * Custom hook for form management with validation
 */
export function useForm<T extends Record<string, any>>(
  initialValues: T,
  validationRules?: ValidationRules<T>,
  onSubmit?: (values: T) => void | Promise<void>
) {
  const [values, setValues] = useState<T>(initialValues);
  const [errors, setErrors] = useState<FormErrors<T>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [touched, setTouched] = useState<Record<string, boolean>>({});

  // Handle form input changes
  const handleChange = (
    e: ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>
  ) => {
    const { name, value, type } = e.target;
    const key = name as keyof T;
    const newValue = type === 'checkbox' 
      ? (e.target as HTMLInputElement).checked 
      : value;
    
    setValues({ ...values, [key]: newValue });
    
    // Validate field if rules exist
    if (validationRules && key in validationRules) {
      const validationFn = validationRules[key];
      if (validationFn) {
        // Cast the new value to match the expected type for this field
        const validationResult = validationFn(newValue as T[keyof T]);
        if (validationResult) {
          setErrors({ ...errors, [key]: validationResult });
        } else {
          const newErrors = { ...errors };
          delete newErrors[key];
          setErrors(newErrors);
        }
      }
    }

    // Mark as touched
    setTouched({ ...touched, [key]: true });
  };

  // Handle direct value updates from components like switches or custom inputs
  const setValue = <K extends keyof T>(name: K, value: T[K]) => {
    setValues({ ...values, [name]: value });
    
    // Validate field if rules exist
    if (validationRules && name in validationRules) {
      const validationFn = validationRules[name];
      if (validationFn) {
        const validationResult = validationFn(value);
        if (validationResult) {
          setErrors({ ...errors, [name]: validationResult });
        } else {
          const newErrors = { ...errors };
          delete newErrors[name];
          setErrors(newErrors);
        }
      }
    }

    // Mark as touched
    setTouched({ ...touched, [name]: true });
  };

  // Handle form submission
  const handleSubmit = async (e?: FormEvent) => {
    if (e) e.preventDefault();
    
    // Validate all fields
    let isValid = true;
    const newErrors: FormErrors<T> = {};
    const allTouched: Record<string, boolean> = {};

    if (validationRules) {
      // Mark all fields as touched during submission
      Object.keys(values).forEach((key) => {
        allTouched[key] = true;
      });
      setTouched(allTouched);

      // Validate all fields
      for (const key in validationRules) {
        const typedKey = key as keyof T;
        const validationFn = validationRules[typedKey];
        if (validationFn) {
          const validationResult = validationFn(values[typedKey]);
          if (validationResult) {
            newErrors[typedKey] = validationResult;
            isValid = false;
          }
        }
      }
    }
    
    setErrors(newErrors);
    
    if (isValid && onSubmit) {
      setIsSubmitting(true);
      try {
        await onSubmit(values);
      } catch (error) {
        console.error('Form submission error:', error);
      } finally {
        setIsSubmitting(false);
      }
    }
    
    return isValid;
  };

  // Reset form to initial state or new values
  const reset = (newValues?: T) => {
    setValues(newValues || initialValues);
    setErrors({});
    setTouched({});
    setIsSubmitting(false);
  };

  return {
    values,
    errors,
    touched,
    isSubmitting,
    handleChange,
    setValue,
    handleSubmit,
    reset
  };
}