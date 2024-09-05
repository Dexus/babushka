use crate::point::Point2D;
use crate::polygon::Polygon;
use crate::segment::Segment;
use approx::abs_diff_eq;
use itertools::Itertools;
use num_traits::{Float, Zero};

#[derive(Debug)]
enum TouchingType {
    A,
    B,
    C,
}

#[derive(Debug)]
struct Touching {
    tt: TouchingType,
    a: usize,
    b: usize,
}

#[derive(Clone, Copy, Debug)]
enum PolygonSource {
    A,
    B,
}

#[derive(Clone, Copy, Debug)]
struct Vector<P> {
    point: P,
    start: usize,
    end: usize,
    source: PolygonSource,
}
pub trait ComputeNoFitPolygon: Polygon {
    /// Return the vertex at the given index after transformations.
    fn get_vertex(&self, index: usize) -> <Self as Polygon>::Point;

    fn no_fit_polygon(
        &self,
        other: &Self,
        inside: bool,
        search_edges: bool,
    ) -> Option<Vec<Vec<<Self as Polygon>::Point>>> {
        // we will be mucking with the offset of other so clone it
        let mut other = other.clone();
        other.set_offset(Zero::zero());

        // keep track of visited vertices
        let mut self_marked = vec![false; self.length()];
        let mut other_marked = vec![false; other.length()];

        let min_self_by_y = self
            .iter_vertices_local()
            .min_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap()
            .clone();

        let max_other_by_y = other
            .iter_vertices_local()
            .max_by(|a, b| a.y().partial_cmp(&b.y()).unwrap())
            .unwrap()
            .clone();


        let mut start_point = if !inside {
            Some(min_self_by_y - max_other_by_y)
        } else {
            self.search_start_point(&other, &self_marked, true, None)
        };


        let mut nfp_list = vec![];

        while let Some(current_start_point) = start_point {
            other.set_offset(current_start_point);

            // Touching Type, A index, B index
            let mut touchings: Vec<Touching>;
            let mut prev_vector = None::<Vector<<Self as Polygon>::Point>>;
            let mut nfp: Option<Vec<<Self as Polygon>::Point>> = Some(vec![other.get_vertex(0)]);

            let mut reference = other.get_vertex(0);
            println!("reference: {:?}", reference);
            let start = reference;
            let mut counter = 0;

            // Sanity check, prevent infinite loop
            while counter < 10 * (self.length() + other.length()) {
                touchings = vec![];

                // find touching vertices / edges
                // we need to carry around indices into self and other
                // to avoid dealing with lots of mutable refernces
                for ((idx_self_start, self_segment), (idx_other_start, other_segment)) in self
                    .iter_segments_local()
                    .enumerate()
                    .cartesian_product(other.iter_segments().enumerate())
                {
                    let idx_self_end = if idx_self_start == self.length() {
                        0
                    } else {
                        idx_self_start + 1
                    };
                    let idx_other_end = if idx_other_start == other.length() {
                        0
                    } else {
                        idx_other_start + 1
                    };

                    if abs_diff_eq!(self_segment.start(), other_segment.start()) {
                        touchings.push(Touching {
                            tt: TouchingType::A,
                            a: idx_self_start,
                            b: idx_other_start,
                        });
                    } else if other_segment.start().on_segment(&self_segment) {
                        touchings.push(Touching {
                            tt: TouchingType::B,
                            a: idx_self_end,
                            b: idx_other_start,
                        });
                    } else if self_segment.start().on_segment(&other_segment) {
                        touchings.push(Touching {
                            tt: TouchingType::C,
                            a: idx_self_start,
                            b: idx_other_end,
                        });
                    }
                }

                // generate translation vectors from touching vertices / edges
                let mut vectors: Vec<Vector<<Self as Polygon>::Point>> = vec![];
                for touching in touchings {
                    let vertex_self = self.get_vertex(touching.a);
                    self_marked[touching.a] = true;

                    // adjacent self vertices
                    let prev_self_index = if touching.a == 0 {
                        self.length() - 1
                    } else {
                        touching.a - 1
                    };
                    let next_self_index = if touching.a == self.length() - 1 {
                        0
                    } else {
                        touching.a + 1
                    };

                    let prev_vertex_self = self.get_vertex(prev_self_index);
                    let next_vertex_self = self.get_vertex(next_self_index);

                    // adjacent B vertices
                    let vertex_other = other.get_vertex(touching.b);
                    let prev_other_index = if touching.b == 0 {
                        other.length() - 1
                    } else {
                        touching.b - 1
                    };
                    let next_other_index = if touching.b == other.length() - 1 {
                        0
                    } else {
                        touching.b + 1
                    };
                    let prev_vertex_other = other.get_vertex(prev_other_index);
                    let next_vertex_other = other.get_vertex(next_other_index);

                    match touching.tt {
                        TouchingType::A => {
                            vectors.push(Vector {
                                point: prev_vertex_self - vertex_self,
                                start: touching.a,
                                end: prev_self_index,
                                source: PolygonSource::A,
                            });
                            vectors.push(Vector {
                                point: next_vertex_self - vertex_self,
                                start: touching.a,
                                end: next_self_index,
                                source: PolygonSource::A,
                            });

                            // other's vectors need to be inverted
                            // TODO: check if we need to actually localize the other polygon
                            vectors.push(Vector {
                                point: vertex_other - prev_vertex_other,// - other.offset(),
                                start: prev_other_index,
                                end: touching.b,
                                source: PolygonSource::B,
                            });
                            vectors.push(Vector {
                                point: vertex_other - next_vertex_other,// - other.offset(),
                                start: next_other_index,
                                end: touching.b,
                                source: PolygonSource::B,
                            });
                        }
                        TouchingType::B => {
                            vectors.push(Vector {
                                point: vertex_self - vertex_other,
                                start: prev_self_index,
                                end: touching.a,
                                source: PolygonSource::A,
                            });
                            vectors.push(Vector {
                                point: prev_vertex_self - vertex_self,
                                start: touching.a,
                                end: prev_self_index,
                                source: PolygonSource::A,
                            });
                        }
                        TouchingType::C => {
                            vectors.push(Vector {
                                point: vertex_self - vertex_other,
                                start: prev_other_index,
                                end: touching.b,
                                source: PolygonSource::B,
                            });
                            vectors.push(Vector {
                                point: vertex_self - prev_vertex_other,
                                start: touching.b,
                                end: prev_other_index,
                                source: PolygonSource::B,
                            });
                        }
                    }
                }
                let mut translate = None::<Vector<<Self as Polygon>::Point>>;
                let mut max_d = <<Self as Polygon>::Point as Point2D>::Value::zero();

                println!("prev_vector: {:?}", prev_vector);
                for vector in vectors {
                    if vector.point.is_zero() {
                        continue;
                    }

                    // if this vector points us back to where we came from, ignore it.
                    // ie cross product = 0 and dot product < 0
                    if let Some(prev_vector) = &prev_vector {
                        if prev_vector.point.dot(&vector.point) < Zero::zero() {
                            // compare magnitude with unit vectors
                            let vector_unit = vector.point.normalized().unwrap();
                            let prev_unit = prev_vector.point.normalized().unwrap();

                            if (vector_unit.y() * prev_unit.x() - vector_unit.x() * prev_unit.y())
                                .abs()
                                < Float::epsilon()
                            {
                                continue;
                            }
                        }
                    }

                    let mut d = self.slide_distance_on_polygon(&other, vector.point);
                    let vector_d2 = vector.point.dot(&vector.point);
                    println!("vector: {:?}, slide_distance: {:?}", vector, d);

                    if d.is_none() || d.unwrap() * d.unwrap() > vector_d2 {
                        d = Some(vector_d2.sqrt());
                    }

                    if let Some(d) = d {
                        if d > max_d {
                            max_d = d;
                            translate = Some(vector);
                            println!("translate: {:?}", translate);
                        }
                    }
                }

                if translate.is_none() || abs_diff_eq!(max_d, Zero::zero()) {
                    // didn't close the loop, something went wrong here
                    println!("Something went wrong, didn't close the loop, translate: {:?}, max_d: {:?}", translate, max_d);
                    nfp = None;
                    break;
                }
                if let Some(translate) = &translate {
                    match translate.source {
                        PolygonSource::A => {
                            self_marked[translate.start] = true;
                            self_marked[translate.end] = true;
                        }
                        PolygonSource::B => {
                            other_marked[translate.start] = true;
                            other_marked[translate.end] = true;
                        }
                    }
                }
                prev_vector = translate;

                // trim
                let vector_length_2 = translate.unwrap().point.dot(&translate.unwrap().point);
                if max_d * max_d < vector_length_2 && !abs_diff_eq!(max_d * max_d, vector_length_2)
                {
                    let scale = ((max_d * max_d) / vector_length_2).sqrt();
                    translate = translate.map(|mut translate| {
                        translate.point.set_x(translate.point.x() * scale);
                        translate.point.set_y(translate.point.y() * scale);
                        translate
                    });
                }

                reference.set_x(reference.x() + translate.unwrap().point.x());
                reference.set_y(reference.y() + translate.unwrap().point.y());

                // we've made a full loop
                println!("reference: {:?}", reference);
                if abs_diff_eq!(reference, start) {
                    println!("we've made a full loop");
                    break;
                }

                // if self and other start on a touching horizontal line,
                // the end point may not be the start point
                let mut looped = false;

                if let Some(nfp) = &nfp {
                    if !nfp.is_empty() {
                        for i in 0..nfp.len() - 1 {
                            if abs_diff_eq!(reference, nfp[i]) {
                                looped = true;
                            }
                        }
                    }
                }

                if looped {
                    break;
                }

                if let Some(nfp) = nfp.as_mut() {
                    nfp.push(reference);
                }

                other.set_offset(other.offset() + translate.unwrap().point);

                counter += 1;
            }

            if let Some(nfp) = nfp {
                if !nfp.is_empty() {
                    nfp_list.push(nfp);
                }
            }

            if !search_edges {
                break;
            }

            start_point =
                self.search_start_point(&other, &self_marked, inside, Some(nfp_list.clone()))
        }

        Some(nfp_list)
    }

    fn search_start_point(
        &self,
        other: &Self,
        self_marked: &Vec<bool>,
        inside: bool,
        nfp: Option<Vec<Vec<<Self as Polygon>::Point>>>,
    ) -> Option<<Self as Polygon>::Point> {
        //let self_clone = self.clone();
        let mut other = other.clone();
        let mut self_marked = self_marked.clone();

        // since we are iterating over every segment, the index i will be the index of
        // the starting point of that segment
        for (i, self_segment) in self.iter_segments().enumerate() {
            if !self_marked[i] {
                self_marked[i] = true;

                for j in 0..other.length() {
                    other.set_offset(*self_segment.start() - other.get_vertex(j));

                    // TODO: This kinda looks suspicious
                    let mut other_inside = None::<bool>;
                    for k in 0..other.length() {
                        if let Some(in_poly) = other.get_vertex(k).in_polygon(self) {
                            other_inside = Some(in_poly);
                            break;
                        }
                    }

                    // A and B are the same
                    let Some(mut other_inside) = other_inside else {
                        return None;
                    };

                    let mut start_point = other.offset();
                    if (other_inside && inside || !other_inside && !inside)
                        && self.intersects_polygon(&other)
                        && !Self::in_nfp(&start_point, &nfp)
                    {
                        return Some(start_point);
                    }

                    // Slide other along vector
                    let mut v = *self_segment.end() - *self_segment.start();
                    let d1 = self.project_distance_on_polygon(&other, v);
                    let d2 = self.project_distance_on_polygon(&other, -v);

                    let d = if d1.is_none() && d2.is_none() {
                        None
                    } else if d1.is_none() {
                        d2
                    } else if d2.is_none() {
                        d1
                    } else {
                        Some(d1.unwrap().min(d2.unwrap()))
                    };

                    // only slide until no longer negative
                    let Some(d) = d else {
                        continue;
                    };
                    if !(!abs_diff_eq!(d, Zero::zero()) && d > Zero::zero()) {
                        continue;
                    }

                    let vd2 = v.dot(&v);
                    if d * d < vd2 && !abs_diff_eq!(d * d, vd2) {
                        let vd = v.dot(&v);
                        v.set_x(v.x() * d / vd);
                        v.set_y(v.y() * d / vd);
                    }

                    other.set_offset(other.offset() + v);

                    // TODO: This kinda looks suspicious
                    for k in 0..other.length() {
                        if let Some(in_poly) = other.get_vertex(k).in_polygon(self) {
                            other_inside = in_poly;
                            break;
                        }
                    }
                    start_point = other.offset();
                    if (other_inside && inside || !other_inside && !inside)
                        && self.intersects_polygon(&other)
                        && !Self::in_nfp(&start_point, &nfp)
                    {
                        return Some(start_point);
                    }
                }
            }
        }

        None
    }

    fn in_nfp(
        p: &<Self as Polygon>::Point,
        nfp: &Option<Vec<Vec<<Self as Polygon>::Point>>>,
    ) -> bool {
        let Some(nfp) = nfp else {
            return false;
        };

        if nfp.is_empty() {
            return false;
        }

        for poly in nfp {
            for point in poly {
                if abs_diff_eq!(p.x(), point.x()) && abs_diff_eq!(p.y(), point.y()) {
                    return true;
                }
            }
        }

        false
    }
}
