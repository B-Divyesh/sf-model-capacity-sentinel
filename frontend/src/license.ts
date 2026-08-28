const slug='model-capacity-sentinel';
const tokenKey=`sb_license:${slug}`;const verdictKey=`${tokenKey}:verdict`;
export type LicenseState={unlocked:boolean;checking:boolean;notice:string;token:string};
export const checkoutUrl=`https://api.sociobot.in/api/v1/products/${slug}/checkout`;
export function consumeLicenseFromUrl(){const url=new URL(location.href);const token=url.searchParams.get('license');if(token){localStorage.setItem(tokenKey,token);url.searchParams.delete('license');history.replaceState({},'',url.pathname+url.search+url.hash)}return token}
export function storedToken(){return localStorage.getItem(tokenKey)||''}
export function storeToken(token:string){localStorage.setItem(tokenKey,token.trim());localStorage.removeItem(verdictKey)}
export async function verifyLicense(token:string,force=false):Promise<LicenseState>{
  if(!token)return{unlocked:false,checking:false,notice:'',token:''};const cached=JSON.parse(localStorage.getItem(verdictKey)||'null') as {valid:boolean;at:number}|null;
  const day=86_400_000;if(!force&&cached&&Date.now()-cached.at<day)return{unlocked:cached.valid,checking:false,notice:cached.valid?'Atlas license active':'License no longer active',token};
  try{const res=await fetch(`https://api.sociobot.in/api/v1/products/${slug}/verify?license=${encodeURIComponent(token)}`);const body=await res.json();const valid=Boolean(body.valid);localStorage.setItem(verdictKey,JSON.stringify({valid,at:Date.now()}));return{unlocked:valid,checking:false,notice:valid?'Atlas license active':'License no longer active',token}}
  catch{return{unlocked:cached?.valid??false,checking:false,notice:'License check will retry when online',token}}
}
