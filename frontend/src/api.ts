import type { ProbeInput, Summary, Observation } from './types';

export class ApiError extends Error { constructor(public status:number,message:string){super(message)} }
let accessToken='';
export function setAccessToken(token:string){accessToken=token}
async function request<T>(path:string, init?:RequestInit):Promise<T>{
  const response=await fetch(path,{...init,headers:{'Content-Type':'application/json',...(accessToken?{Authorization:`Bearer ${accessToken}`} : {}),...init?.headers}});
  if(!response.ok){let message=`Request failed (${response.status})`;try{const body=await response.json();message=body.error||message}catch{}throw new ApiError(response.status,message)}
  if(response.status===204)return undefined as T;return response.json();
}
export const api={
  summary:()=>request<Summary>('/api/summary'),
  create:(body:ProbeInput)=>request<{id:string}>('/api/probes',{method:'POST',body:JSON.stringify(body)}),
  update:(id:string,body:ProbeInput)=>request<{id:string}>(`/api/probes/${id}`,{method:'PUT',body:JSON.stringify(body)}),
  remove:(id:string)=>request<void>(`/api/probes/${id}`,{method:'DELETE'}),
  run:(id:string)=>request<Observation>(`/api/probes/${id}/run`,{method:'POST'}),
  exportCsv:async()=>{
    const response=await fetch('/api/export.csv',{headers:accessToken?{Authorization:`Bearer ${accessToken}`}:{}});
    if(!response.ok)throw new ApiError(response.status,'Could not export observations');
    return response.blob();
  }
};
