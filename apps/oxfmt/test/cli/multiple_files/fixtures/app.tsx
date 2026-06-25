interface   Props   {
    title  :  string
    count  :  number
}
const   App   :   React.FC  <  Props  >   =   (  {  title  ,  count  }  )   =>   {
return   <div  >
        <h1  >  {  title  }  </h1>
    <span  >  Count  :  {  count  }  </span>
</div>
}
