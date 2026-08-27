import React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger';
}

export function Button({ children, variant = 'primary', className = '', ...props }: ButtonProps) {
  const baseStyle = "px-4 py-2 rounded-md font-semibold transition-all duration-300 transform active:scale-95";
  let variantStyle = "";
  
  if (variant === 'primary') {
    variantStyle = "bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white shadow-lg hover:shadow-indigo-500/20";
  } else if (variant === 'secondary') {
    variantStyle = "bg-slate-800 hover:bg-slate-700 text-white border border-slate-700";
  } else if (variant === 'danger') {
    variantStyle = "bg-red-600 hover:bg-red-700 text-white shadow-lg hover:shadow-red-500/20";
  }

  return (
    <button className={`${baseStyle} ${variantStyle} ${className}`} {...props}>
      {children}
    </button>
  );
}
