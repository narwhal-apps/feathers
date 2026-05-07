import { redirect } from '@sveltejs/kit';

export function load({ params }: { params: { id: string } }) {
  throw redirect(307, `/repo/${params.id}/changes/`);
}
