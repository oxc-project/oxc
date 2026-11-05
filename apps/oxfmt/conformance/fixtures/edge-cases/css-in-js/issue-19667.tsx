import React, { useEffect, useRef } from 'react'

const typographyStyle = {
  '14_400': css`
  font-size: 14px;
    line-height: 16px;
  `
}


const DummyComponent: React.FC = () => {
  const ref = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (ref.current) ref.current.focus()
  }, [])

  return (
    <div>{Boolean(ref.current) ?? (
      <input type="text" ref={ref} className={css`

                &&& .ant-dropdown-menu-item {
                  padding: 6px 8px !important;
                  margin-left: 4px;
                  margin-right: 4px;
                }

                &&& .ant-dropdown-menu-item-selected {
                  background-color: white;
                }

                &&& .ant-dropdown-menu-title-content {
                  font-size: 14px;
                  ${typographyStyle['14_400']};
                }
      `} />
    )}
    </div>
  )
}

export default DummyComponent
