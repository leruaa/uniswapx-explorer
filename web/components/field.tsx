import { ReactNode } from "react"
import clsx from 'clsx';

export default ({ title, titleClassName, contentClassName, children }: { title: string, titleClassName?: string, contentClassName?: string, children: ReactNode }) => {

  return (
    <div className="flex">
      <h3 className={clsx("w-64", titleClassName)}>{title}</h3>
      <div className={contentClassName}>{children}</div>
    </div>
  )
}