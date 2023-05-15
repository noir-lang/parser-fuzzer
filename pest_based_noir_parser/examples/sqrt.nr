fn sqrt(x: Field) -> Field {
    let mut z = pow(x, C3);
    let mut t = z*z*x;
    z *= x;
    let mut b = t;
    let mut c = C5;
 
    for i in 0..(C1-1) {
        
        for _j in 1..(C1-i-1) {
            
            b *= b;
            
        }
 
        z *= if b == 1 { 1 } else { c };
 
        c *= c;
 
        t *= if b == 1 { 1 } else { c };
 
        b = t;
    }
    
    z
}